use crunch_postgres::PostgresPersistence;

#[tokio::test]
async fn test_new_from_env() -> anyhow::Result<()> {
    PostgresPersistence::new_from_env().await?;

    Ok(())
}
