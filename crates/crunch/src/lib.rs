mod traits {
    use std::fmt::Display;

    use async_trait::async_trait;

    use crate::{DeserializeError, SerializeError};

    #[async_trait]
    pub trait Persistence {
        async fn insert(&self, event_info: &EventInfo, content: Vec<u8>) -> anyhow::Result<()>;
    }

    pub trait Serializer {
        fn serialize(&self) -> Result<Vec<u8>, SerializeError>;
    }

    pub trait Deserializer {
        fn deserialize(raw: Vec<u8>) -> Result<Self, DeserializeError>
        where
            Self: Sized;
    }

    #[derive(Debug, Clone, Copy)]
    pub struct EventInfo {
        pub domain: &'static str,
        pub entity_type: &'static str,
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
        fn event_info(&self) -> EventInfo;
    }
}

mod impls {
    use std::{ops::Deref, sync::Arc};

    use async_trait::async_trait;

    use crate::{traits, EventInfo};

    pub struct InMemoryPersistence {}

    #[async_trait]
    impl traits::Persistence for InMemoryPersistence {
        async fn insert(&self, event_info: &EventInfo, content: Vec<u8>) -> anyhow::Result<()> {
            tracing::info!(
                event_info = event_info.to_string(),
                content_len = content.len(),
                "inserted event"
            );

            Ok(())
        }
    }

    pub struct Persistence(Arc<dyn traits::Persistence + Send + Sync + 'static>);

    impl Persistence {
        pub fn in_memory() -> Self {
            Self(Arc::new(InMemoryPersistence {}))
        }
    }

    impl Deref for Persistence {
        type Target = Arc<dyn traits::Persistence + Send + Sync + 'static>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
}

mod errors {
    use thiserror::Error;

    #[derive(Error, Debug)]
    pub enum SerializeError {
        #[error("failed to serialize")]
        FailedToSerialize(anyhow::Error),
    }

    #[derive(Error, Debug)]
    pub enum DeserializeError {
        #[error("failed to serialize")]
        FailedToDeserialize(anyhow::Error),
    }

    #[derive(Error, Debug)]
    pub enum PublishError {
        #[error("failed to serialize")]
        SerializeError(#[source] SerializeError),

        #[error("failed to commit to database")]
        DbError(#[source] anyhow::Error),

        #[error("transaction failed")]
        DbTxError(#[source] anyhow::Error),

        #[error("failed to connect to database")]
        ConnectionError(#[source] anyhow::Error),
    }
}

pub use errors::*;
pub use impls::Persistence;
pub use traits::{Deserializer, Event, EventInfo, Serializer};

pub struct Publisher {
    persistence: Persistence,
}

#[allow(dead_code)]
impl Publisher {
    pub fn new(persistence: Persistence) -> Self {
        Self { persistence }
    }

    pub async fn publish<T>(&self, event: T) -> Result<(), PublishError>
    where
        T: Event,
    {
        let content = event.serialize().map_err(PublishError::SerializeError)?;

        self.persistence
            .insert(&event.event_info(), content)
            .await
            .map_err(PublishError::DbError)?;

        Ok(())
    }
    pub async fn publish_tx<T>(&self, event: T) -> Result<(), PublishError>
    where
        T: Event,
    {
        // TODO: add transaction support later
        self.publish(event).await
    }
}
