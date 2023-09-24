use thiserror::Error;

#[derive(Error, Debug)]
pub enum SerializeError {
    #[error("failed to serialize {0}")]
    FailedToSerialize(anyhow::Error),
}

#[derive(Error, Debug)]
pub enum DeserializeError {
    #[error("failed to deserialize {0}")]
    FailedToDeserialize(anyhow::Error),
    #[error("failed to deserialize {0}")]
    ProtoErr(prost::DecodeError),
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
pub enum SubscriptionError {
    #[error("failed to subscribe: {0}")]
    FailedToSubscribe(#[source] anyhow::Error),

    #[error("connection failed: {0}")]
    ConnectionFailed(#[source] TransportError),

    #[error("failed to deserialize{0}")]
    DeserializationFailed(#[source] DeserializeError),
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

#[derive(Error, Debug)]
pub enum BuilderError {
    #[error("dependency not added to builder: {0}")]
    DependencyError(anyhow::Error),
}
