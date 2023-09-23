use crunch_traits::DynTransport;

#[derive(Clone)]
pub struct Transport(DynTransport);

impl Transport {
    pub fn new(transport: DynTransport) -> Self {
        Self(transport)
    }

    #[cfg(feature = "in-memory")]
    pub fn in_memory() -> Self {
        Self(std::sync::Arc::new(
            crunch_in_memory::InMemoryTransport::default(),
        ))
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
