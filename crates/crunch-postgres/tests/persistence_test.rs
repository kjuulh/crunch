use crunch_postgres::PostgresPersistence;
use crunch_traits::{EventInfo, Persistence};

#[tokio::test]
async fn test_persistence_insert() -> anyhow::Result<()> {
    let persistence = PostgresPersistence::new_from_env().await?;

    persistence
        .insert(
            &EventInfo {
                domain: "some-domain",
                entity_type: "some-entity-type",
                event_name: "some-event-name",
            },
            b"some-strange-and-cruncy-content".to_vec(),
        )
        .await?;

    persistence
        .insert(
            &EventInfo {
                domain: "some-domain",
                entity_type: "some-entity-type",
                event_name: "some-event-name",
            },
            b"some-strange-and-cruncy-content".to_vec(),
        )
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_persistence_next() -> anyhow::Result<()> {
    let persistence = PostgresPersistence::new_from_env().await?;

    persistence
        .insert(
            &EventInfo {
                domain: "some-domain",
                entity_type: "some-entity-type",
                event_name: "some-event-name",
            },
            b"some-strange-and-cruncy-content".to_vec(),
        )
        .await?;

    persistence
        .insert(
            &EventInfo {
                domain: "some-domain",
                entity_type: "some-entity-type",
                event_name: "some-event-name",
            },
            b"some-strange-and-cruncy-content".to_vec(),
        )
        .await?;

    assert!(persistence.next().await?.is_some());
    assert!(persistence.next().await?.is_some());

    Ok(())
}
