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
    let counter = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));

    let inner_counter = counter.clone();
    crunch
        .subscribe(move |item: SomeEvent| {
            let counter = inner_counter.clone();

            async move {
                tracing::debug!(
                    "subscription got event: {}, info: {}",
                    item.name,
                    item.int_event_info(),
                );

                counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok(())
            }
        })
        .await?;

    let event = SomeEvent {
        name: "something".into(),
    };

    // Publish a lot of events
    for _ in 0..50 {
        tokio::spawn({
            let event = event.clone();
            let crunch = crunch.clone();

            async move {
                loop {
                    crunch.publish(event.clone()).await.unwrap();
                    tokio::time::sleep(std::time::Duration::from_millis(1)).await;
                }
            }
        });
    }

    tokio::time::sleep(std::time::Duration::from_secs(30)).await;

    let amount_run = counter.load(std::sync::atomic::Ordering::SeqCst);
    tracing::info!("ran {} amount of times", amount_run);

    Ok(())
}
