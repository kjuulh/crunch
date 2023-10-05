use crunch::errors::*;
use crunch::traits::Event;

#[derive(Clone)]
struct SomeEvent {
    name: String,
}

impl crunch::traits::Serializer for SomeEvent {
    fn serialize(&self) -> Result<Vec<u8>, SerializeError> {
        Ok(b"field=name".to_vec())
    }
}

impl crunch::traits::Deserializer for SomeEvent {
    fn deserialize(_raw: Vec<u8>) -> Result<Self, DeserializeError>
    where
        Self: Sized,
    {
        Ok(Self {
            name: "something".into(),
        })
    }
}

impl crunch::traits::Event for SomeEvent {
    fn event_info() -> crunch::traits::EventInfo {
        crunch::traits::EventInfo {
            domain: "some-domain".into(),
            entity_type: "some-entity".into(),
            event_name: "some-event".into(),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let crunch = crunch::Builder::default().build()?;
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

    crunch.publish(event.clone()).await?;
    crunch.publish(event.clone()).await?;
    crunch.publish(event.clone()).await?;

    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    Ok(())
}
