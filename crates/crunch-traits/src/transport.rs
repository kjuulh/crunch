use std::sync::Arc;

use async_trait::async_trait;

use crate::{errors::TransportError, EventInfo};

#[async_trait]
pub trait Transport {
    async fn publish(&self, event_info: &EventInfo, content: Vec<u8>)
        -> Result<(), TransportError>;
}
pub type DynTransport = Arc<dyn Transport + Send + Sync + 'static>;
