use crate::tcp_config::TcpConfig;
use crate::error::ConnectionError;
use crate::handshake::{HandshakeOpcode, HandshakeResponse};
use crate::macros::with_shutdown;
use crate::request::{FileRequest, RequestOpcode};
use crate::response::WorldListEncoder;
use crate::file_service::FileService;
use std::io::ErrorKind;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::TcpStream;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::mpsc;

const CLIENT_VERSION: u32 = 592;

enum ClientState {
    Handshake,
    JS5,
    WorldList,
    Login,
}

enum ReaderMessage {
    FileRequest(FileRequest),
    EncryptionKey(u8),
}

pub struct Connection {
    reader: BufReader<OwnedReadHalf>,
    writer: BufWriter<OwnedWriteHalf>,
    service: Arc<FileService>,
    config: TcpConfig,
}

impl Connection {
    pub fn new(socket: TcpStream, service: Arc<FileService>, config: TcpConfig) -> Self {
        let (read_half, write_half) = socket.into_split();
        Self {
            reader: BufReader::new(read_half),
            writer: BufWriter::new(write_half),
            service,
            config,
        }
    }

    pub async fn accept(mut self) -> anyhow::Result<(), ConnectionError> {
        let mut state = ClientState::Handshake;

        loop {
            match state {
                ClientState::Handshake => {
                    state = self.handle_handshake().await?;
                }
                ClientState::JS5 => return self.handle_js5().await,
                ClientState::WorldList => {
                    return with_shutdown!(self.writer, self.handle_worldlist().await);
                }
                ClientState::Login => {
                    return with_shutdown!(self.writer, self.handle_login().await);
                }
            }
        }
    }

    async fn handle_handshake(&mut self) -> anyhow::Result<ClientState, ConnectionError> {
        let opcode = self.reader.read_u8().await?;

        match HandshakeOpcode::from_byte(opcode) {
            Some(HandshakeOpcode::Js5) => {
                let client_version = self.reader.read_u32().await?;
                let response = if client_version == CLIENT_VERSION {
                    HandshakeResponse::Success
                } else {
                    HandshakeResponse::OutOfDate
                };

                self.writer.write_u8(response.as_byte()).await?;
                self.writer.flush().await?;

                if response != HandshakeResponse::Success {
                    return Err(ConnectionError::VersionMismatch);
                }

                Ok(ClientState::JS5)
            }

            Some(HandshakeOpcode::WorldList) => Ok(ClientState::WorldList),
            Some(HandshakeOpcode::Login) => Ok(ClientState::Login),
            None => Err(ConnectionError::InvalidHandshakeOpcode(opcode)),
        }
    }

    async fn handle_js5(self) -> anyhow::Result<(), ConnectionError> {
        let buffer_size = self.config.request_buffer_size;
        let (urgent_tx, urgent_rx) = mpsc::channel(buffer_size);
        let (normal_tx, normal_rx) = mpsc::channel(buffer_size);
        let (key_tx, key_rx) = mpsc::channel(1);
        let reader_handle =
            tokio::spawn(Self::reader_task(self.reader, urgent_tx, normal_tx, key_tx));
        let writer_handle = tokio::spawn(Self::writer_task(
            self.writer,
            urgent_rx,
            normal_rx,
            key_rx,
            self.service,
        ));

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

    async fn reader_task(
        mut reader: BufReader<OwnedReadHalf>,
        urgent_tx: mpsc::Sender<FileRequest>,
        normal_tx: mpsc::Sender<FileRequest>,
        key_tx: mpsc::Sender<u8>,
    ) -> anyhow::Result<(), ConnectionError> {
        loop {
            match Self::read_message(&mut reader).await? {
                Some(ReaderMessage::FileRequest(request)) => {
                    let tx = if request.urgent {
                        &urgent_tx
                    } else {
                        &normal_tx
                    };

                    if tx.send(request).await.is_err() {
                        return Ok(());
                    }
                }
                Some(ReaderMessage::EncryptionKey(encryption_key)) => {
                    let _ = key_tx.send(encryption_key).await;
                }
                None => return Ok(()),
            }
        }
    }

    async fn read_message(
        reader: &mut BufReader<OwnedReadHalf>,
    ) -> anyhow::Result<Option<ReaderMessage>, ConnectionError> {
        loop {
            let opcode = match reader.read_u8().await {
                Ok(b) => b,
                Err(e) if e.kind() == ErrorKind::UnexpectedEof => return Ok(None),
                Err(e) if e.kind() == ErrorKind::ConnectionReset => return Ok(None),
                Err(e) => return Err(e.into()),
            };

            let request_opcode = RequestOpcode::from_byte(opcode);
            match request_opcode {
                Some(RequestOpcode::FileRequestNormal) | Some(RequestOpcode::FileRequestUrgent) => {
                    let mut data = [0u8; 3];
                    reader.read_exact(&mut data).await?;
                    let urgent = request_opcode == Some(RequestOpcode::FileRequestUrgent);
                    let request = FileRequest::parse(urgent, &data);
                    return Ok(Some(ReaderMessage::FileRequest(request)));
                }
                Some(RequestOpcode::EncryptionKey) => {
                    let key = reader.read_u8().await?;
                    reader.read_exact(&mut [0u8; 2]).await?;
                    return Ok(Some(ReaderMessage::EncryptionKey(key)));
                }
                Some(_) => {
                    reader.read_exact(&mut [0u8; 3]).await?;
                }
                None => return Err(ConnectionError::InvalidRequestOpcode(opcode)),
            }
        }
    }

    async fn writer_task(
        mut writer: BufWriter<OwnedWriteHalf>,
        mut urgent_rx: mpsc::Receiver<FileRequest>,
        mut normal_rx: mpsc::Receiver<FileRequest>,
        mut key_rx: mpsc::Receiver<u8>,
        service: Arc<FileService>,
    ) -> anyhow::Result<(), ConnectionError> {
        let mut xor_key: u8 = 0;

        loop {
            tokio::select! {
                biased;

                Some(key) = key_rx.recv() => xor_key = key,
                Some(request) = urgent_rx.recv() => {
                    if let Ok(response) = service.serve(&request) {
                        Self::write_xor(&mut writer, &response, xor_key).await?;
                    }
                }
                Some(request) = normal_rx.recv() => {
                    if let Ok(response) = service.serve(&request) {
                        Self::write_xor(&mut writer, &response, xor_key).await?;
                    }
                }
                else => {
                    let _ = writer.shutdown().await;
                    return Ok(())
                }
            }
        }
    }

    async fn write_xor(
        writer: &mut BufWriter<OwnedWriteHalf>,
        data: &[u8],
        key: u8,
    ) -> anyhow::Result<(), ConnectionError> {
        if key == 0 {
            writer.write_all(data).await?;
        } else {
            let encoded: Vec<u8> = data.iter().map(|b| b ^ key).collect();
            writer.write_all(&encoded).await?;
        }
        writer.flush().await?;
        Ok(())
    }

    async fn handle_worldlist(&mut self) -> anyhow::Result<(), ConnectionError> {
        let full_update = self.reader.read_u8().await? == 0;
        let response = WorldListEncoder::encode(full_update, "127.0.0.1", 100);

        self.writer.write_all(&response).await?;
        self.writer.flush().await?;

        Ok(())
    }

    async fn handle_login(&mut self) -> anyhow::Result<(), ConnectionError> {
        unimplemented!()
    }
}
