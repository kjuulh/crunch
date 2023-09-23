use thiserror::Error;

#[derive(Error, Debug)]
pub enum SerializeError {
    #[error("failed to serialize {0}")]
    FailedToSerialize(anyhow::Error),
}

#[derive(Error, Debug)]
pub enum DeserializeError {
    #[error("failed to serialize {0}")]
    FailedToDeserialize(anyhow::Error),
}

#[derive(Error, Debug)]
pub enum PublishError {
    #[error("failed to serialize {0}")]
    SerializeError(#[source] SerializeError),

    #[error("failed to commit to database {0}")]
    DbError(#[source] anyhow::Error),

    #[error("transaction failed {0}")]
    DbTxError(#[source] anyhow::Error),

    #[error("failed to connect to database {0}")]
    ConnectionError(#[source] anyhow::Error),
}

#[derive(Error, Debug)]
pub enum TransportError {
    #[error("to publish to transport {0}")]
    Err(anyhow::Error),
}

#[derive(Error, Debug)]
pub enum PersistenceError {
    #[error("failed to get item {0}")]
    GetErr(anyhow::Error),

    #[error("failed to publish item {0}")]
    UpdatePublished(anyhow::Error),
}
