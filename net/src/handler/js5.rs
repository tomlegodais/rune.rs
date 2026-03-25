use crate::codec::{Js5Codec, XorCodec};
use crate::error::SessionError;
use crate::message::{Js5Inbound, Js5Outbound, PriorityRequest};
use crate::service::CacheService;
use futures_util::{SinkExt, StreamExt};
use std::collections::BinaryHeap;
use std::future::poll_fn;
use std::sync::Arc;
use std::task::Poll;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

pub struct Js5Handler;

impl Js5Handler {
    pub async fn run(
        stream: TcpStream,
        service: Arc<CacheService>,
    ) -> anyhow::Result<(), SessionError> {
        let codec = Js5Codec::default();
        let xor_codec = XorCodec::new(codec);
        let mut framed = Framed::new(stream, xor_codec);
        let mut queue: BinaryHeap<PriorityRequest> = BinaryHeap::new();
        let mut sequence: u64 = 0;

        loop {
            match Self::recv(&mut framed, queue.is_empty()).await {
                Some(Ok(msg)) => Self::read(msg, &mut framed, &mut queue, &mut sequence),
                Some(Err(e)) => return Err(e),
                None if queue.is_empty() => return Ok(()),
                None => {}
            }

            Self::process(&mut framed, &mut queue, &service).await?;
        }
    }

    async fn recv(
        framed: &mut Framed<TcpStream, XorCodec<Js5Codec>>,
        blocking: bool,
    ) -> Option<Result<Js5Inbound, SessionError>> {
        if blocking {
            return framed.next().await;
        }

        poll_fn(|cx| {
            Poll::Ready(match framed.poll_next_unpin(cx) {
                Poll::Ready(msg) => msg,
                Poll::Pending => None,
            })
        })
        .await
    }

    fn read(
        msg: Js5Inbound,
        framed: &mut Framed<TcpStream, XorCodec<Js5Codec>>,
        queue: &mut BinaryHeap<PriorityRequest>,
        sequence: &mut u64,
    ) {
        match msg {
            Js5Inbound::FileRequest(request) => {
                queue.push(PriorityRequest {
                    request,
                    sequence: *sequence,
                });
                *sequence += 1;
            }

            Js5Inbound::EncryptionKey(key) => framed.codec_mut().set_xor_key(key),
        }
    }

    async fn process(
        framed: &mut Framed<TcpStream, XorCodec<Js5Codec>>,
        queue: &mut BinaryHeap<PriorityRequest>,
        service: &Arc<CacheService>,
    ) -> anyhow::Result<(), SessionError> {
        let Some(priority) = queue.pop() else {
            return Ok(());
        };

        let request = priority.request;
        let service = Arc::clone(service);
        let file_result = tokio::task::spawn_blocking(move || {
            service.get_file(&request).ok().map(|data| Js5Outbound {
                index: request.index,
                archive: request.archive,
                data,
                urgent: request.urgent,
            })
        })
        .await
        .ok()
        .flatten();

        if let Some(outbound) = file_result {
            framed.send(outbound).await?;
        }

        Ok(())
    }
}
