use crunch_traits::DynTransport;

#[derive(Clone)]
pub struct Transport(DynTransport);

impl Transport {
    pub fn new(transport: DynTransport) -> Self {
        Self(transport)
    }

    #[cfg(feature = "in-memory")]
    pub fn in_memory() -> Self {
        use crunch_in_memory::transport::InMemoryTransport;

        Self(std::sync::Arc::new(InMemoryTransport::default()))
    }

    #[cfg(feature = "nats")]
    pub async fn nats(
        options: crate::nats::NatsConnectOptions<'_>,
    ) -> Result<Self, crunch_traits::errors::TransportError> {
        Ok(Self(std::sync::Arc::new(
            crunch_nats::NatsTransport::new(options).await?,
        )))
    }

    #[cfg(feature = "nodata")]
    pub fn nodata(host: &str) -> Result<Self, crunch_traits::errors::TransportError> {
        Ok(Self(std::sync::Arc::new(
            crunch_nodata::NoDataTransport::new(host),
        )))
    }
}

impl From<DynTransport> for Transport {
    fn from(value: DynTransport) -> Self {
        Self::new(value)
    }
}

impl std::ops::Deref for Transport {
    type Target = DynTransport;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
