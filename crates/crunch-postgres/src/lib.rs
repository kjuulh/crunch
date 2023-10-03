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
    domain: &'static str,
    entity_type: &'static str,
    event_name: &'static str,
}

impl From<&EventInfo> for PgEventInfo {
    fn from(value: &EventInfo) -> Self {
        Self {
            domain: value.domain,
            entity_type: value.entity_type,
            event_name: value.event_name,
        }
    }
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

        let id = match resp {
            Some(InsertResp { id }) => Some(id.to_string()),
            None => None,
        };

        Ok(id.map(|id| (id, Box::new(PostgresTx {}) as crunch_traits::DynTx)))
    }
    async fn get(&self, event_id: &str) -> Result<Option<(EventInfo, Vec<u8>)>, PersistenceError> {
        todo!()
    }
    async fn update_published(&self, event_id: &str) -> Result<(), PersistenceError> {
        todo!()
    }
}
