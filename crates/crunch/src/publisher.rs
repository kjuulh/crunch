use crunch_traits::{errors::PublishError, Event};

use crate::Persistence;

#[derive(Clone)]
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
            .insert(&event.int_event_info(), content)
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
