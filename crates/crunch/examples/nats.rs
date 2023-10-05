use crunch::errors::*;
use crunch::nats::{NatsConnectCredentials, NatsConnectOptions};
use crunch::traits::{Deserializer, Event, EventInfo, Serializer};

#[derive(Clone)]
struct SomeEvent {
    name: String,
}

impl Serializer for SomeEvent {
    fn serialize(&self) -> Result<Vec<u8>, SerializeError> {
        Ok(b"field=name".to_vec())
    }
}

impl Deserializer for SomeEvent {
    fn deserialize(_raw: Vec<u8>) -> Result<Self, DeserializeError>
    where
        Self: Sized,
    {
        Ok(Self {
            name: "something".into(),
        })
    }
}

impl Event for SomeEvent {
    fn event_info() -> EventInfo {
        EventInfo {
            domain: "some-domain".into(),
            entity_type: "some-entity".into(),
            event_name: "some-event".into(),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    // Remember to start nats via. docker first, or set equivalent settings as in `NatsConnectOptions`
    let crunch = crunch::Builder::default()
        .with_nats_transport(NatsConnectOptions {
            host: "127.0.0.1:4222",
            credentials: NatsConnectCredentials::UserPass {
                user: "user",
                pass: "secret",
            },
        })
        .await?
        .build()?;
    crunch
        .subscribe(move |item: SomeEvent| async move {
            tracing::info!(
                "subscription got event: {}, info: {}",
                item.name,
                item.int_event_info(),
            );
            Ok(())
        })
        .await?;

    let event = SomeEvent {
        name: "something".into(),
    };

    for _ in 0..5 {
        crunch.publish(event.clone()).await?;
    }

    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    Ok(())
}
