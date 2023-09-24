use std::{
    collections::{BTreeMap, VecDeque},
    ops::Deref,
    sync::Arc,
};

use async_trait::async_trait;
use crunch_traits::{errors::PersistenceError, EventInfo};
use tokio::sync::RwLock;

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
    store: Arc<RwLock<BTreeMap<String, Msg>>>,
}

#[async_trait]
impl crunch_traits::Persistence for InMemoryPersistence {
    async fn insert(&self, event_info: &EventInfo, content: Vec<u8>) -> anyhow::Result<()> {
        let msg = crunch_envelope::proto::wrap(event_info.domain, event_info.entity_type, &content);
        let msg = Msg {
            id: uuid::Uuid::new_v4().to_string(),
            info: *event_info,
            msg,
            state: MsgState::Pending,
        };
        let mut outbox = self.outbox.write().await;
        outbox.push_back(msg.clone());
        self.store.write().await.insert(msg.id.clone(), msg);

        tracing::debug!(
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

    async fn get(&self, event_id: &str) -> Result<Option<(EventInfo, Vec<u8>)>, PersistenceError> {
        let store = self.store.read().await;

        let event = match store.get(event_id).filter(|m| m.state == MsgState::Pending) {
            Some(event) => event,
            None => return Ok(None),
        };

        let (content, _) = crunch_envelope::proto::unwrap(event.msg.as_slice())
            .map_err(|e| PersistenceError::GetErr(anyhow::anyhow!(e)))?;

        Ok(Some((event.info, content)))
    }

    async fn update_published(&self, event_id: &str) -> Result<(), PersistenceError> {
        match self.store.write().await.get_mut(event_id) {
            Some(msg) => msg.state = MsgState::Published,
            None => {
                return Err(PersistenceError::UpdatePublished(anyhow::anyhow!(
                    "event was not found on id: {}",
                    event_id
                )))
            }
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct Persistence {
    inner: Arc<dyn crunch_traits::Persistence + Send + Sync + 'static>,
}

impl Persistence {
    #[cfg(feature = "in-memory")]
    pub fn in_memory() -> Self {
        Self {
            inner: std::sync::Arc::new(InMemoryPersistence {
                outbox: std::sync::Arc::default(),
                store: std::sync::Arc::default(),
            }),
        }
    }
}

impl Deref for Persistence {
    type Target = Arc<dyn crunch_traits::Persistence + Send + Sync + 'static>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
