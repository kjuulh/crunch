use crunch::{Deserializer, Event, EventInfo, Persistence, Publisher, Serializer};

struct SomeEvent {
    name: String,
}

impl Serializer for SomeEvent {
    fn serialize(&self) -> Result<Vec<u8>, crunch::SerializeError> {
        Ok(b"field=name".to_vec())
    }
}

impl Deserializer for SomeEvent {
    fn deserialize(raw: Vec<u8>) -> Result<Self, crunch::DeserializeError>
    where
        Self: Sized,
    {
        Ok(Self {
            name: "something".into(),
        })
    }
}

impl Event for SomeEvent {
    fn event_info(&self) -> EventInfo {
        EventInfo {
            domain: "some-domain",
            entity_type: "some-entity",
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let in_memory = Persistence::in_memory();
    let publisher = Publisher::new(in_memory);

    publisher
        .publish(SomeEvent {
            name: "something".into(),
        })
        .await?;

    Ok(())
}
