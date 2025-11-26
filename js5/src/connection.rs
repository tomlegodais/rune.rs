use crate::error::ConnectionError;
use crate::handshake::{HandshakeOpcode, HandshakeResponse};
use crate::config::Js5Config;
use crate::service::Js5Service;
use crate::request::RequestOpcode::*;
use crate::request::{FileRequest, RequestOpcode};
use std::io::ErrorKind;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
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

        let (tx, rx) = mpsc::channel(self.config.request_buffer_size);
        let (reader, writer) = self.socket.into_split();
        let reader = BufReader::new(reader);
        let writer = BufWriter::new(writer);
        let reader_handle = tokio::spawn(Self::reader_task(reader, tx));
        let writer_handle = tokio::spawn(Self::writer_task(writer, rx, self.service));

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

    async fn handshake(&mut self) -> anyhow::Result<(), ConnectionError> {
        let opcode = self.socket.read_u8().await?;
        match HandshakeOpcode::from_byte(opcode) {
            Some(HandshakeOpcode::Js5) => {}
            Some(HandshakeOpcode::Login) => return Err(ConnectionError::WrongService),
            None => return Err(ConnectionError::InvalidHandshake),
        }

        let client_version = self.socket.read_u32().await?;
        let response = if client_version == self.config.version {
            HandshakeResponse::Success
        } else {
            HandshakeResponse::OutOfDate
        };

        self.socket.write_u8(response.as_byte()).await?;
        self.socket.flush().await?;

        if response != HandshakeResponse::Success {
            return Err(ConnectionError::VersionMismatch);
        }

        Ok(())
    }

    async fn reader_task(
        mut reader: BufReader<OwnedReadHalf>,
        tx: mpsc::Sender<FileRequest>,
    ) -> anyhow::Result<(), ConnectionError> {
        loop {
            let message = Self::read_message(&mut reader).await?;
            match message {
                Some(ClientMessage::FileRequest(request)) => {
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
            .ok_or_else(|| ConnectionError::InvalidOpcode(opcode))?;

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
                let mut keys = [0u8; 16];
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
        mut rx: mpsc::Receiver<FileRequest>,
        service: Arc<Js5Service>,
    ) -> anyhow::Result<(), ConnectionError> {
        while let Some(request) = rx.recv().await {
            if let Ok(response) = service.serve(&request) {
                writer.write_all(&response).await?;
                writer.flush().await?;
            }
        }
        Ok(())
    }
}
