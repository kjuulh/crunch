use crunch_postgres::PostgresPersistence;
use crunch_traits::{EventInfo, Persistence};

#[tokio::test]
async fn test_persistence_insert() -> anyhow::Result<()> {
    let persistence = PostgresPersistence::new_from_env().await?;

    persistence
        .insert(
            &EventInfo {
                domain: "some-domain".into(),
                entity_type: "some-entity-type".into(),
                event_name: "some-event-name".into(),
            },
            b"some-strange-and-cruncy-content".to_vec(),
        )
        .await?;

    persistence
        .insert(
            &EventInfo {
                domain: "some-domain".into(),
                entity_type: "some-entity-type".into(),
                event_name: "some-event-name".into(),
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
                domain: "some-domain".into(),
                entity_type: "some-entity-type".into(),
                event_name: "some-event-name".into(),
            },
            b"some-strange-and-cruncy-content".to_vec(),
        )
        .await?;

    persistence
        .insert(
            &EventInfo {
                domain: "some-domain".into(),
                entity_type: "some-entity-type".into(),
                event_name: "some-event-name".into(),
            },
            b"some-strange-and-cruncy-content".to_vec(),
        )
        .await?;

    assert!(persistence.next().await?.is_some());
    assert!(persistence.next().await?.is_some());

    Ok(())
}

#[tokio::test]
async fn test_persistence_get() -> anyhow::Result<()> {
    let persistence = PostgresPersistence::new_from_env().await?;

    persistence
        .insert(
            &EventInfo {
                domain: "some-domain".into(),
                entity_type: "some-entity-type".into(),
                event_name: "some-event-name".into(),
            },
            b"some-strange-and-cruncy-content".to_vec(),
        )
        .await?;

    let (event_id, _) = persistence.next().await?.unwrap();
    let (_, _) = persistence.get(&event_id).await?.unwrap();

    Ok(())
}

#[tokio::test]
async fn test_persistence_update() -> anyhow::Result<()> {
    let persistence = PostgresPersistence::new_from_env().await?;

    persistence
        .insert(
            &EventInfo {
                domain: "some-domain".into(),
                entity_type: "some-entity-type".into(),
                event_name: "some-event-name".into(),
            },
            b"some-strange-and-cruncy-content".to_vec(),
        )
        .await?;

    let (event_id, _) = persistence.next().await?.unwrap();
    let (_, _) = persistence.get(&event_id).await?.unwrap();

    persistence.update_published(&event_id).await?;

    Ok(())
}
