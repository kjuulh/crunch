use std::{ops::Deref, sync::Arc};

#[derive(Clone)]
pub struct Persistence {
    inner: Arc<dyn crunch_traits::Persistence + Send + Sync + 'static>,
}

impl Persistence {
    #[cfg(feature = "in-memory")]
    pub fn in_memory() -> Self {
        use crunch_in_memory::persistence::InMemoryPersistence;

        Self {
            inner: std::sync::Arc::new(InMemoryPersistence {
                outbox: std::sync::Arc::default(),
                store: std::sync::Arc::default(),
            }),
        }
    }
}

impl Deref for Persistence {
    type Target = Arc<dyn crunch_traits::Persistence + Send + Sync + 'static>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
