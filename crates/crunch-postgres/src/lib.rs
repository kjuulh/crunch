use async_trait::async_trait;
use crunch_traits::{errors::PersistenceError, EventInfo};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, types::Json, Pool, Postgres};
use uuid::Uuid;

pub struct PostgresTx {}

impl crunch_traits::Tx for PostgresTx {}

pub struct PostgresPersistence {
    pool: Pool<Postgres>,
}

impl PostgresPersistence {
    pub async fn new(dsn: &str) -> anyhow::Result<Self> {
        let pool = PgPoolOptions::new().max_connections(5).connect(dsn).await?;

        sqlx::migrate!().run(&pool).await?;

        Ok(Self { pool })
    }

    pub async fn new_from_env() -> anyhow::Result<Self> {
        let dsn = std::env::var("DATABASE_URL")
            .map_err(|e| anyhow::anyhow!("DATABASE_URL is not set: {e}"))?;

        Self::new(&dsn).await
    }
}

#[derive(sqlx::FromRow)]
struct InsertResp {
    id: Uuid,
}

#[derive(Clone, Serialize, Deserialize)]
struct PgEventInfo {
    domain: String,
    entity_type: String,
    event_name: String,
}

impl From<&EventInfo> for PgEventInfo {
    fn from(value: &EventInfo) -> Self {
        let value = value.to_owned();

        Self {
            domain: value.domain,
            entity_type: value.entity_type,
            event_name: value.event_name,
        }
    }
}

impl From<PgEventInfo> for EventInfo {
    fn from(value: PgEventInfo) -> Self {
        EventInfo {
            domain: value.domain,
            entity_type: value.entity_type,
            event_name: value.event_name,
        }
    }
}

#[allow(dead_code)]
#[derive(sqlx::FromRow)]
struct OutboxEvent {
    id: Uuid,
    metadata: Json<PgEventInfo>,
    content: Vec<u8>,
    inserted_time: chrono::DateTime<chrono::Utc>,
    state: String,
}

#[async_trait]
impl crunch_traits::Persistence for PostgresPersistence {
    // FIXME: Need some sort of concurrency control mechanism. If the insert fails or is done twice, then that user may receive multiple requests.
    // This should be solved by adding transactions, event streams and sequence numbers
    async fn insert(&self, event_info: &EventInfo, content: Vec<u8>) -> anyhow::Result<()> {
        let event_info: PgEventInfo = event_info.into();
        sqlx::query_as::<_, InsertResp>(
            r#"
INSERT INTO outbox (id, metadata, content, state) 
VALUES (
    $1, 
    $2, 
    $3, 
    'inserted'
) 
RETURNING id;
"#,
        )
        .bind(uuid::Uuid::new_v4())
        .bind(Json(&event_info))
        .bind(content)
        .fetch_one(&self.pool)
        .await?;

        Ok(())
    }
    async fn next(&self) -> Result<Option<(String, crunch_traits::DynTx)>, PersistenceError> {
        let resp = sqlx::query_as::<_, InsertResp>(
            r#"
SELECT id 
FROM outbox 
WHERE state = 'inserted' 
ORDER BY inserted_time ASC 
LIMIT 1 
FOR UPDATE;
"#,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!(e))
        .map_err(PersistenceError::AnyErr)?;

        let id = resp.map(|InsertResp { id }| id.to_string());

        Ok(id.map(|id| (id, Box::new(PostgresTx {}) as crunch_traits::DynTx)))
    }
    async fn get(&self, event_id: &str) -> Result<Option<(EventInfo, Vec<u8>)>, PersistenceError> {
        let event = sqlx::query_as::<_, OutboxEvent>("SELECT * from outbox where id = $1")
            .bind(
                Uuid::parse_str(event_id)
                    .map_err(|e| anyhow::anyhow!(e))
                    .map_err(PersistenceError::GetErr)?,
            )
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| anyhow::anyhow!(e))
            .map_err(PersistenceError::GetErr)?;

        match event {
            Some(event) => {
                let metadata = event.metadata.to_owned();

                Ok(Some((EventInfo::from(metadata.0), event.content)))
            }
            None => Ok(None),
        }
    }
    async fn update_published(&self, event_id: &str) -> Result<(), PersistenceError> {
        todo!()
    }
}
