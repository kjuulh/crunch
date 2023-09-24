mod gencrunch;

use gencrunch::basic::{includes::my_include::MyInclude, my_event::MyEvent};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let crunch = ::crunch::builder::Builder::default().build()?;

    crunch
        .subscribe(|item: MyEvent| async move {
            println!("received item: {:?}", item);

            Ok(())
        })
        .await?;

    crunch
        .publish(MyEvent {
            name: "some-name".into(),
            include: Some(MyInclude {
                name: "some-name".into(),
            }),
        })
        .await?;

    // Sleep a while to let subscriber catch item
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    Ok(())
}
