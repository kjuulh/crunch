mod errors;
mod impls;
mod traits;

pub use errors::*;
pub use impls::Persistence;
pub use traits::{Deserializer, Event, EventInfo, Serializer};

mod outbox {
    use std::sync::Arc;

    pub use crate::Persistence;

    pub struct OutboxHandler {
        persistence: Persistence,
    }

    impl OutboxHandler {
        pub fn new(persistence: Persistence) -> Self {
            Self { persistence }
        }

        pub async fn spawn(&mut self) {
            let p = self.persistence.clone();
            tokio::spawn(async move {
                let p = p;

                while let Some(item) = p.next().await {}
            });
        }
    }
}

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
