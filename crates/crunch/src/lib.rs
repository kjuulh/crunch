mod impls;
mod outbox;
mod publisher;
mod subscriber;
mod transport;

#[cfg(feature = "traits")]
pub mod traits {
    pub use crunch_traits::{Deserializer, Event, EventInfo, Persistence, Serializer, Transport};
}

pub mod errors {
    pub use crunch_traits::errors::*;
}

use crunch_traits::Event;
pub use impls::Persistence;
pub use outbox::OutboxHandler;
pub use publisher::Publisher;
pub use subscriber::Subscriber;
pub use transport::Transport;

#[derive(Clone)]
pub struct Crunch {
    publisher: Publisher,
    subscriber: Subscriber,
}
impl Crunch {
    pub fn new(publisher: Publisher, subscriber: Subscriber) -> Self {
        Self {
            publisher,
            subscriber,
        }
    }

    pub async fn subscribe<I, F, Fut>(&self, callback: F) -> Result<(), errors::SubscriptionError>
    where
        F: Fn(I) -> Fut + Send + Sync + 'static,
        Fut: futures::Future<Output = Result<(), errors::SubscriptionError>> + Send + 'static,
        I: Event + Send + 'static,
    {
        self.subscriber.subscribe(callback).await
    }
}

impl std::ops::Deref for Crunch {
    type Target = Publisher;

    fn deref(&self) -> &Self::Target {
        &self.publisher
    }
}

pub mod builder {
    use crate::{errors, Crunch, OutboxHandler, Persistence, Publisher, Subscriber, Transport};

    #[derive(Clone)]
    pub struct Builder {
        persistence: Option<Persistence>,
        transport: Option<Transport>,
        outbox_enabled: bool,
    }

    impl Builder {
        #[cfg(feature = "in-memory")]
        pub fn with_in_memory_persistence(&mut self) -> &mut Self {
            self.persistence = Some(Persistence::in_memory());
            self
        }

        #[cfg(feature = "in-memory")]
        pub fn with_in_memory_transport(&mut self) -> &mut Self {
            self.transport = Some(Transport::in_memory());
            self
        }

        pub fn with_outbox(&mut self, enabled: bool) -> &mut Self {
            self.outbox_enabled = enabled;
            self
        }

        pub fn build(&mut self) -> Result<Crunch, errors::BuilderError> {
            let persistence =
                self.persistence
                    .clone()
                    .ok_or(errors::BuilderError::DependencyError(anyhow::anyhow!(
                        "persistence was not set"
                    )))?;
            let transport = self
                .transport
                .clone()
                .ok_or(errors::BuilderError::DependencyError(anyhow::anyhow!(
                    "transport was not set"
                )))?;

            let publisher = Publisher::new(persistence.clone());
            let subscriber = Subscriber::new(transport.clone());
            if self.outbox_enabled {
                OutboxHandler::new(persistence.clone(), transport.clone()).spawn();
            }

            Ok(Crunch::new(publisher, subscriber))
        }
    }

    impl Default for Builder {
        fn default() -> Self {
            #[cfg(feature = "in-memory")]
            {
                return Self {
                    outbox_enabled: true,
                    persistence: None,
                    transport: None,
                }
                .with_in_memory_persistence()
                .with_in_memory_transport()
                .clone();
            }

            Self {
                persistence: None,
                transport: None,
                outbox_enabled: true,
            }
        }
    }
}
