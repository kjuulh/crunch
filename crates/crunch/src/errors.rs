use thiserror::Error;

#[derive(Error, Debug)]
pub enum SerializeError {
    #[error("failed to serialize")]
    FailedToSerialize(anyhow::Error),
}

#[derive(Error, Debug)]
pub enum DeserializeError {
    #[error("failed to serialize")]
    FailedToDeserialize(anyhow::Error),
}

#[derive(Error, Debug)]
pub enum PublishError {
    #[error("failed to serialize")]
    SerializeError(#[source] SerializeError),

    #[error("failed to commit to database")]
    DbError(#[source] anyhow::Error),

    #[error("transaction failed")]
    DbTxError(#[source] anyhow::Error),

    #[error("failed to connect to database")]
    ConnectionError(#[source] anyhow::Error),
}
