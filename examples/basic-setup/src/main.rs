mod crunch;

use ::crunch::traits::{Deserializer, Event, EventInfo, Serializer};

struct MyEvent {}

impl Serializer for MyEvent {
    fn serialize(&self) -> Result<Vec<u8>, ::crunch::errors::SerializeError> {
        todo!()
    }
}
impl Deserializer for MyEvent {
    fn deserialize(_raw: Vec<u8>) -> Result<Self, ::crunch::errors::DeserializeError>
    where
        Self: Sized,
    {
        todo!()
    }
}

impl Event for MyEvent {
    fn event_info() -> ::crunch::traits::EventInfo {
        EventInfo {
            domain: "my-domain",
            entity_type: "my-entity-type",
            event_name: "my-event-name",
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    crunch::basic::my_event::MyEvent {
        name: "some-name".into(),
        include: Some(crunch::basic::includes::my_include::MyInclude {
            name: "some-name".into(),
        }),
    };

    let crunch = ::crunch::builder::Builder::default().build()?;

    crunch
        .subscribe(|_item: MyEvent| async move { Ok(()) })
        .await?;

    crunch.publish(MyEvent {}).await?;

    Ok(())
}
