use std::fmt::Display;

use async_trait::async_trait;
use errors::{DeserializeError, PersistenceError, SerializeError};

pub trait Tx: Send + Sync {}

pub type DynTx = Box<dyn Tx>;

#[async_trait]
pub trait Persistence {
    async fn insert(&self, event_info: &EventInfo, content: Vec<u8>) -> anyhow::Result<()>;
    async fn next(&self) -> Result<Option<(String, DynTx)>, PersistenceError>;
    async fn get(&self, event_id: &str) -> Result<Option<(EventInfo, Vec<u8>)>, PersistenceError>;
    async fn update_published(&self, event_id: &str) -> Result<(), PersistenceError>;
}

pub trait Serializer {
    fn serialize(&self) -> Result<Vec<u8>, SerializeError>;
}

pub trait Deserializer {
    fn deserialize(raw: Vec<u8>) -> Result<Self, DeserializeError>
    where
        Self: Sized;
}

#[derive(Debug, Clone)]
pub struct EventInfo {
    pub domain: String,
    pub entity_type: String,
    pub event_name: String,
}

impl Display for EventInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "domain: {}, entity_type: {}",
            self.domain, self.entity_type
        ))
    }
}

pub trait Event: Serializer + Deserializer {
    fn event_info() -> EventInfo;

    fn int_event_info(&self) -> EventInfo {
        Self::event_info()
    }
}

pub mod errors;
mod transport;
pub use transport::*;
