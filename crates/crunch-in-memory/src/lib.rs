use std::collections::BTreeMap;

use async_trait::async_trait;
use crunch_traits::{errors::TransportError, EventInfo, Transport};
use tokio::sync::broadcast::{Receiver, Sender};

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
}

impl Default for InMemoryTransport {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Transport for InMemoryTransport {
    async fn publish(
        &self,
        event_info: &EventInfo,
        content: Vec<u8>,
    ) -> Result<(), TransportError> {
        let transport_key = event_info.transport_name();

        // Possibly create a register handle instead, as this requires a write and then read. It may not matter for in memory though
        {
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
