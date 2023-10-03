use async_trait::async_trait;
use crunch_traits::{errors::PersistenceError, EventInfo};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

pub struct PostgresPersistence {
    pool: Pool<Postgres>,
}

impl PostgresPersistence {
    pub async fn new(dsn: &str) -> anyhow::Result<Self> {
        let pool = PgPoolOptions::new().max_connections(5).connect(dsn).await?;

        sqlx::migrate!().run(&pool).await?;

        Ok(Self { pool })
    }
}

#[async_trait]
impl crunch_traits::Persistence for PostgresPersistence {
    async fn insert(&self, event_info: &EventInfo, content: Vec<u8>) -> anyhow::Result<()> {
        todo!()
    }
    async fn next(&self) -> Option<String> {
        todo!()
    }
    async fn get(&self, event_id: &str) -> Result<Option<(EventInfo, Vec<u8>)>, PersistenceError> {
        todo!()
    }
    async fn update_published(&self, event_id: &str) -> Result<(), PersistenceError> {
        todo!()
    }
}
