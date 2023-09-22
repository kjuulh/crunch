use std::{collections::VecDeque, future::Future, ops::Deref, pin::Pin, sync::Arc, task::Poll};

use async_trait::async_trait;
use tokio::sync::{Mutex, OnceCell, RwLock};
use tokio_stream::Stream;

use crate::{traits, EventInfo};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum MsgState {
    Pending,
    Published,
}

#[derive(Debug, Clone)]
struct Msg {
    id: String,
    info: EventInfo,
    msg: Vec<u8>,
    state: MsgState,
}

pub struct InMemoryPersistence {
    outbox: Arc<RwLock<VecDeque<Msg>>>,
}

#[async_trait]
impl traits::Persistence for InMemoryPersistence {
    async fn insert(&self, event_info: &EventInfo, content: Vec<u8>) -> anyhow::Result<()> {
        let msg = crunch_envelope::proto::wrap(event_info.domain, event_info.entity_type, &content);

        let mut outbox = self.outbox.write().await;

        outbox.push_back(Msg {
            id: uuid::Uuid::new_v4().to_string(),
            info: event_info.clone(),
            msg,
            state: MsgState::Pending,
        });

        tracing::info!(
            event_info = event_info.to_string(),
            content_len = content.len(),
            "inserted event"
        );

        Ok(())
    }

    async fn next(&self) -> Option<String> {
        let mut outbox = self.outbox.write().await;
        outbox.pop_front().map(|i| i.id)
    }
}

#[derive(Clone)]
pub struct Persistence {
    inner: Arc<dyn traits::Persistence + Send + Sync + 'static>,
    pending: Arc<Option<Pin<Box<dyn Future<Output = Option<OnceCell<String>>> + Send + Sync>>>>,
}

impl Persistence {
    pub fn in_memory() -> Self {
        Self {
            inner: Arc::new(InMemoryPersistence {
                outbox: Arc::default(),
            }),
            pending: Arc::default(),
        }
    }
}

impl Deref for Persistence {
    type Target = Arc<dyn traits::Persistence + Send + Sync + 'static>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Stream for Persistence {
    type Item = OnceCell<String>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let mut pending = self.pending;

        if pending.is_none() {
            *pending = Some(Box::pin(self.inner.next()));
        }

        let fut = pending.as_mut().unwrap();

        match fut.as_mut().poll(cx) {
            Poll::Ready(v) => {
                *pending = None;
                Poll::Ready(v)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
