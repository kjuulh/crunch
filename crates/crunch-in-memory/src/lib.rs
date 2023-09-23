use std::{collections::BTreeMap, pin::Pin};

use async_trait::async_trait;
use crunch_traits::{errors::TransportError, EventInfo, Transport};
use futures::Stream;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio_stream::{wrappers::BroadcastStream, StreamExt};

#[derive(Clone)]
struct TransportEnvelope {
    info: EventInfo,
    content: Vec<u8>,
}

pub struct InMemoryTransport {
    events: tokio::sync::RwLock<BTreeMap<String, Sender<TransportEnvelope>>>,
}

impl InMemoryTransport {
    pub fn new() -> Self {
        Self {
            events: tokio::sync::RwLock::default(),
        }
    }

    async fn register_channel(&self, event_info: &EventInfo) {
        let transport_key = event_info.transport_name();

        // Possibly create a trait register handle instead, as this requires a write and then read. It may not matter for in memory though
        let mut events = self.events.write().await;
        if let None = events.get(&transport_key) {
            let (sender, mut receiver) = tokio::sync::broadcast::channel(100);
            events.insert(transport_key.clone(), sender);
            tokio::spawn(async move {
                while let Ok(item) = receiver.recv().await {
                    tracing::info!("default receiver: {}", item.info.transport_name());
                }
            });
        }
    }
}

impl Default for InMemoryTransport {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Transport for InMemoryTransport {
    type Stream = Pin<Box<dyn Stream<Item = Vec<u8>> + Send>>;

    async fn publish(
        &self,
        event_info: &EventInfo,
        content: Vec<u8>,
    ) -> Result<(), TransportError> {
        self.register_channel(event_info).await;

        let transport_key = event_info.transport_name();
        let events = self.events.read().await;
        let sender = events
            .get(&transport_key)
            .expect("transport to be available, as we just created it");
        sender
            .send(TransportEnvelope {
                info: event_info.clone(),
                content,
            })
            .map_err(|e| anyhow::anyhow!(e.to_string()))
            .map_err(TransportError::Err)?;

        Ok(())
    }

    async fn subscriber(
        &self,
        event_info: &EventInfo,
    ) -> Result<Option<Self::Stream>, TransportError> {
        self.register_channel(event_info).await;

        let events = self.events.read().await;
        match events.get(&event_info.transport_name()) {
            Some(rx) => Ok(Some(Box::pin(
                BroadcastStream::new(rx.subscribe()).filter_map(|m| match m {
                    Ok(m) => Some(m.content),
                    Err(_) => None,
                }),
            ))),
            None => Ok(None),
        }
    }
}

trait EventInfoExt {
    fn transport_name(&self) -> String;
}

impl EventInfoExt for EventInfo {
    fn transport_name(&self) -> String {
        format!(
            "crunch.{}.{}.{}",
            self.domain, self.entity_type, self.event_name
        )
    }
}
