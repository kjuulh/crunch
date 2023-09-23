use crunch_traits::{Event};
use futures::StreamExt;

use crate::{errors, Transport};

#[derive(Clone)]
pub struct Subscriber {
    transport: Transport,
}

impl Subscriber {
    pub fn new(transport: Transport) -> Self {
        Self { transport }
    }

    pub async fn subscribe<I, F, Fut>(&self, callback: F) -> Result<(), errors::SubscriptionError>
    where
        F: Fn(I) -> Fut + Send + Sync + 'static,
        Fut: futures::Future<Output = Result<(), errors::SubscriptionError>> + Send + 'static,
        I: Event + Send + 'static,
    {
        let mut stream = self
            .transport
            .subscriber(&I::event_info())
            .await
            .map_err(errors::SubscriptionError::ConnectionFailed)?
            .ok_or(errors::SubscriptionError::FailedToSubscribe(
                anyhow::anyhow!("failed to find channel to subscribe to"),
            ))?;

        tokio::spawn(async move {
            while let Some(item) = stream.next().await {
                let item = match I::deserialize(item)
                    .map_err(errors::SubscriptionError::DeserializationFailed)
                {
                    Ok(i) => i,
                    Err(e) => {
                        tracing::warn!("deserialization failed: {}", e);
                        continue;
                    }
                };

                match callback(item).await {
                    Ok(_) => {}
                    Err(e) => {
                        tracing::error!("subscription callback failed: {}", e)
                    }
                }
            }
        });

        Ok(())
    }
}
