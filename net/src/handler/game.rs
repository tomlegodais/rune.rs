use crate::codec::GameCodec;
use crate::crypto::StreamCipher;
use crate::{Frame, IncomingMessage, SessionError};
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_util::codec::Framed;

pub struct GameHandler;

impl GameHandler {
    pub async fn run<CIn, COut>(
        stream: TcpStream,
        in_cipher: CIn,
        out_cipher: COut,
        inbox_tx: mpsc::Sender<IncomingMessage>,
        mut outbox_rx: mpsc::Receiver<Frame>,
    ) -> Result<(), SessionError>
    where
        CIn: StreamCipher + 'static,
        COut: StreamCipher + 'static,
    {
        let mut framed = Framed::new(stream, GameCodec::new(in_cipher, out_cipher));

        loop {
            tokio::select! {
                maybe_msg = framed.next() => {
                    let msg = match maybe_msg {
                        Some(Ok(msg)) => msg,
                        Some(Err(e)) => return Err(e),
                        None => break,
                    };

                    if let Some(decoded) = crate::inbound::decode(msg)
                        && inbox_tx.send(decoded).await.is_err()
                    {
                        break;
                    }
                }

                maybe_out = outbox_rx.recv() => {
                    match maybe_out {
                        Some(msg) => framed.send(msg).await?,
                        None => break,
                    }
                }
            }
        }

        Ok(())
    }
}
