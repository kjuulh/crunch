use std::{pin::Pin, sync::Arc};

use async_trait::async_trait;

use crate::{errors::TransportError, EventInfo};

#[async_trait]
pub trait Transport {
    type Stream: futures::Stream<Item = Vec<u8>>;

    async fn publish(&self, event_info: &EventInfo, content: Vec<u8>)
        -> Result<(), TransportError>;
    async fn subscriber(
        &self,
        event_info: &EventInfo,
    ) -> Result<Option<Self::Stream>, TransportError>;
}

pub type DynTransport = Arc<
    dyn Transport<Stream = Pin<Box<dyn futures::Stream<Item = Vec<u8>> + Send>>>
        + Send
        + Sync
        + 'static,
>;
