use std::{collections::VecDeque, ops::Deref, sync::Arc};

use async_trait::async_trait;
use tokio::sync::RwLock;

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
}

impl Persistence {
    pub fn in_memory() -> Self {
        Self {
            inner: Arc::new(InMemoryPersistence {
                outbox: Arc::default(),
            }),
        }
    }
}

impl Deref for Persistence {
    type Target = Arc<dyn traits::Persistence + Send + Sync + 'static>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
