use std::pin::Pin;

use anyhow::Context;
use async_trait::async_trait;
use crunch_traits::{errors::TransportError, EventInfo, Transport};
use futures::{Stream, StreamExt};
use grpc::{no_data_service_client::NoDataServiceClient, PublishEventRequest, SubscribeRequest};
use tonic::transport::{Channel, ClientTlsConfig};

mod grpc;

pub struct NoDataTransport {
    host: String,
}

impl NoDataTransport {
    pub fn new(host: impl Into<String>) -> Self {
        Self { host: host.into() }
    }

    async fn client(&self) -> anyhow::Result<NoDataServiceClient<tonic::transport::Channel>> {
        let channel = if self.host.starts_with("https") {
            Channel::from_shared(self.host.to_owned())
                .context(format!("failed to connect to: {}", &self.host))?
                .tls_config(ClientTlsConfig::new().with_native_roots())?
                .connect()
                .await
                .context(format!("failed to connect to: {}", &self.host))?
        } else {
            Channel::from_shared(self.host.to_owned())
                .context(format!("failed to connect to: {}", &self.host))?
                .connect()
                .await
                .context(format!("failed to connect to: {}", &self.host))?
        };

        let client = NoDataServiceClient::new(channel);

        Ok(client)
    }
}

#[async_trait]
impl Transport for NoDataTransport {
    type Stream = Pin<Box<dyn Stream<Item = Vec<u8>> + Send>>;

    async fn publish(
        &self,
        event_info: &EventInfo,
        content: Vec<u8>,
    ) -> Result<(), TransportError> {
        let mut client = self.client().await.map_err(TransportError::Err)?;

        client
            .publish_event(PublishEventRequest {
                topic: event_info.transport_name(),
                value: content,
            })
            .await
            .context("failed to send crunch(nodata) message")
            .map_err(TransportError::Err)?;

        Ok(())
    }
    async fn subscriber(
        &self,
        event_info: &EventInfo,
    ) -> Result<Option<Self::Stream>, TransportError> {
        let mut client = self.client().await.map_err(TransportError::Err)?;

        let resp_stream = client
            .subscribe(SubscribeRequest {
                topic: event_info.transport_name(),
            })
            .await
            .context("failed to establish connection to nodata")
            .map_err(TransportError::Err)?;

        let sub = resp_stream.into_inner();

        let stream = futures::stream::unfold(sub, |mut sub| async move {
            tracing::trace!("got event from nodata");
            let next = sub.next().await?;

            match next {
                Ok(next) => Some((next.value, sub)),
                Err(e) => {
                    tracing::error!("failed to receive event from nodata: {e}");
                    None
                }
            }
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
