use crunch::errors::*;

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
    fn event_info(&self) -> crunch::traits::EventInfo {
        crunch::traits::EventInfo {
            domain: "some-domain",
            entity_type: "some-entity",
            event_name: "some-event",
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let in_memory = crunch::Persistence::in_memory();
    let transport = crunch::Transport::in_memory();
    crunch::OutboxHandler::new(in_memory.clone(), transport.clone()).spawn();
    let publisher = crunch::Publisher::new(in_memory);

    publisher
        .publish(SomeEvent {
            name: "something".into(),
        })
        .await?;
    publisher
        .publish(SomeEvent {
            name: "something".into(),
        })
        .await?;
    publisher
        .publish(SomeEvent {
            name: "something".into(),
        })
        .await?;
    publisher
        .publish(SomeEvent {
            name: "something".into(),
        })
        .await?;
    publisher
        .publish(SomeEvent {
            name: "something".into(),
        })
        .await?;

    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    Ok(())
}
