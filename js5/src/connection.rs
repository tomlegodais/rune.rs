use crate::config::Js5Config;
use crate::error::ConnectionError;
use crate::handshake::{HandshakeOpcode, HandshakeResponse};
use crate::request::RequestOpcode::*;
use crate::request::{FileRequest, RequestOpcode};
use crate::response::WorldListEncoder;
use crate::service::Js5Service;
use std::io::ErrorKind;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::TcpStream;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::mpsc;

enum ClientMessage {
    FileRequest(FileRequest),
    StateChange(StateChange),
}

#[derive(Debug, Clone, Copy)]
enum StateChange {
    LoggedIn,
    LoggedOut,
    Connected,
    Disconnected,
    EncryptionKeys,
}

pub struct Js5Connection {
    socket: TcpStream,
    service: Arc<Js5Service>,
    config: Js5Config,
}

impl Js5Connection {
    pub fn new(socket: TcpStream, service: Arc<Js5Service>, config: Js5Config) -> Self {
        Js5Connection {
            socket,
            service,
            config,
        }
    }

    pub async fn accept(mut self) -> anyhow::Result<(), ConnectionError> {
        self.socket.set_nodelay(true)?;
        self.handshake().await?;

        let (reader_handle, writer_handle) = self.spawn_io_tasks();

        tokio::select! {
            result = reader_handle => {
                result.map_err(|e| ConnectionError::TaskPanic(e.to_string()))??;
            }
            result = writer_handle => {
                result.map_err(|e| ConnectionError::TaskPanic(e.to_string()))??;
            }
        }

        Ok(())
    }

    fn spawn_io_tasks(
        self,
    ) -> (
        tokio::task::JoinHandle<anyhow::Result<(), ConnectionError>>,
        tokio::task::JoinHandle<anyhow::Result<(), ConnectionError>>,
    ) {
        let buffer_size = self.config.request_buffer_size;
        let (urgent_tx, urgent_rx) = mpsc::channel(buffer_size);
        let (normal_tx, normal_rx) = mpsc::channel(buffer_size);

        let (reader, writer) = self.socket.into_split();

        let reader_handle = Self::spawn_reader_task(reader, urgent_tx, normal_tx);
        let writer_handle = Self::spawn_writer_task(writer, urgent_rx, normal_rx, self.service);

        (reader_handle, writer_handle)
    }

    fn spawn_reader_task(
        reader: OwnedReadHalf,
        urgent_tx: mpsc::Sender<FileRequest>,
        normal_tx: mpsc::Sender<FileRequest>,
    ) -> tokio::task::JoinHandle<anyhow::Result<(), ConnectionError>> {
        let reader = BufReader::new(reader);
        tokio::spawn(Self::reader_task(reader, urgent_tx, normal_tx))
    }

    fn spawn_writer_task(
        writer: OwnedWriteHalf,
        urgent_rx: mpsc::Receiver<FileRequest>,
        normal_rx: mpsc::Receiver<FileRequest>,
        service: Arc<Js5Service>,
    ) -> tokio::task::JoinHandle<anyhow::Result<(), ConnectionError>> {
        let writer = BufWriter::new(writer);
        tokio::spawn(Self::writer_task(writer, urgent_rx, normal_rx, service))
    }

    async fn handshake(&mut self) -> anyhow::Result<(), ConnectionError> {
        let opcode = self.socket.read_u8().await?;

        match HandshakeOpcode::from_byte(opcode) {
            Some(HandshakeOpcode::Js5) => {
                let client_version = self.socket.read_u32().await?;
                let response = match client_version == self.config.version {
                    true => HandshakeResponse::Success,
                    false => HandshakeResponse::OutOfDate,
                };

                self.socket.write_u8(response.as_byte()).await?;
                self.socket.flush().await?;

                if response != HandshakeResponse::Success {
                    return Err(ConnectionError::VersionMismatch);
                }

                Ok(())
            }

            Some(HandshakeOpcode::WorldList) => {
                let full_update = self.socket.read_u8().await? == 0;
                let response = WorldListEncoder::encode(full_update, "127.0.0.1", 1);

                self.socket.write_all(&response).await?;
                self.socket.flush().await?;

                Ok(())
            }
            Some(HandshakeOpcode::Login) => Ok(()),
            None => Err(ConnectionError::InvalidHandshakeOpcode(opcode)),
        }
    }

    async fn reader_task(
        mut reader: BufReader<OwnedReadHalf>,
        urgent_tx: mpsc::Sender<FileRequest>,
        normal_tx: mpsc::Sender<FileRequest>,
    ) -> anyhow::Result<(), ConnectionError> {
        loop {
            let message = Self::read_message(&mut reader).await?;
            match message {
                Some(ClientMessage::FileRequest(request)) => {
                    let tx = if request.urgent {
                        &urgent_tx
                    } else {
                        &normal_tx
                    };
                    if tx.send(request).await.is_err() {
                        return Ok(());
                    }
                }
                Some(ClientMessage::StateChange(s)) => {
                    println!("State change: {:?}", s);
                }
                None => return Ok(()),
            }
        }
    }

    async fn read_message<R: AsyncReadExt + Unpin>(
        reader: &mut R,
    ) -> anyhow::Result<Option<ClientMessage>, ConnectionError> {
        let opcode = match reader.read_u8().await {
            Ok(b) => b,
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => return Ok(None),
            Err(e) => return Err(e.into()),
        };

        let opcode = RequestOpcode::from_byte(opcode)
            .ok_or_else(|| ConnectionError::InvalidRequestOpcode(opcode))?;

        match opcode {
            FileRequestNormal | FileRequestUrgent => {
                let mut data = [0u8; 3];
                reader.read_exact(&mut data).await?;
                let request = FileRequest::parse(opcode == FileRequestUrgent, &data);
                Ok(Some(ClientMessage::FileRequest(request)))
            }

            LoggedIn => {
                reader.read_exact(&mut [0u8; 3]).await?;
                Ok(Some(ClientMessage::StateChange(StateChange::LoggedIn)))
            }

            LoggedOut => {
                reader.read_exact(&mut [0u8; 3]).await?;
                Ok(Some(ClientMessage::StateChange(StateChange::LoggedOut)))
            }

            EncryptionKeys => {
                let mut keys = [0u8; 3];
                reader.read_exact(&mut keys).await?;
                Ok(Some(ClientMessage::StateChange(
                    StateChange::EncryptionKeys,
                )))
            }

            Connected => {
                reader.read_exact(&mut [0u8; 3]).await?;
                Ok(Some(ClientMessage::StateChange(StateChange::Connected)))
            }

            Disconnected => {
                reader.read_exact(&mut [0u8; 3]).await?;
                Ok(Some(ClientMessage::StateChange(StateChange::Disconnected)))
            }
        }
    }

    async fn writer_task(
        mut writer: BufWriter<OwnedWriteHalf>,
        mut urgent_rx: mpsc::Receiver<FileRequest>,
        mut normal_rx: mpsc::Receiver<FileRequest>,
        service: Arc<Js5Service>,
    ) -> anyhow::Result<(), ConnectionError> {
        loop {
            let request = tokio::select! {
                biased;

                urgent_request = urgent_rx.recv() => {
                    match urgent_request {
                        Some(request) => request,
                        None => break,
                    }
                }

                normal_request = normal_rx.recv() => {
                    match normal_request {
                        Some(request) => request,
                        None => break,
                    }
                }
            };

            if let Ok(response) = service.serve(&request) {
                writer.write_all(&response).await?;
                writer.flush().await?;
            }
        }
        Ok(())
    }
}
