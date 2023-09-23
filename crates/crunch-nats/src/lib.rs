use std::pin::Pin;

use async_trait::async_trait;
use crunch_traits::{errors::TransportError, EventInfo, Transport};
use futures::Stream;

pub struct NatsConnectOptions<'a> {
    pub host: &'a str,
    pub credentials: NatsConnectCredentials<'a>,
}
pub enum NatsConnectCredentials<'a> {
    UserPass { user: &'a str, pass: &'a str },
}

#[derive(Clone)]
pub struct NatsTransport {
    conn: nats::asynk::Connection,
}

impl NatsTransport {
    pub async fn new(options: NatsConnectOptions<'_>) -> Result<Self, TransportError> {
        let conn = match options.credentials {
            NatsConnectCredentials::UserPass { user, pass } => {
                nats::asynk::Options::with_user_pass(user, pass)
                    .connect(options.host)
                    .await
                    .map_err(|e| anyhow::anyhow!(e))
                    .map_err(TransportError::Err)?
            }
        };

        Ok(Self { conn })
    }
}

#[async_trait]
impl Transport for NatsTransport {
    type Stream = Pin<Box<dyn Stream<Item = Vec<u8>> + Send>>;

    async fn publish(
        &self,
        event_info: &EventInfo,
        content: Vec<u8>,
    ) -> Result<(), TransportError> {
        self.conn
            .publish(&event_info.transport_name(), &content)
            .await
            .map_err(|e| anyhow::anyhow!(e))
            .map_err(TransportError::Err)
    }
    async fn subscriber(
        &self,
        event_info: &EventInfo,
    ) -> Result<Option<Self::Stream>, TransportError> {
        let sub = self
            .conn
            .subscribe(&event_info.transport_name())
            .await
            .map_err(|e| anyhow::anyhow!(e))
            .map_err(TransportError::Err)?;

        let stream = futures::stream::unfold(sub, |sub| async move {
            let next = sub.next().await?;
            let next = next.data;
            Some((next, sub))
        });

        Ok(Some(Box::pin(stream)))
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
