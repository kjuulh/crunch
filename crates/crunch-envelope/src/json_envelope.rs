use base64::{engine::general_purpose, Engine};
use serde::{Deserialize, Serialize};

use crate::EnvelopeError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Envelope {
    content: String,
    metadata: Metadata,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Metadata {
    domain: String,
    entity: String,
}

pub fn wrap<'a>(domain: &'a str, entity: &'a str, content: &'a [u8]) -> Vec<u8> {
    serde_json::to_vec(&Envelope {
        content: general_purpose::URL_SAFE_NO_PAD.encode(content),
        metadata: Metadata {
            domain: domain.to_string(),
            entity: entity.to_string(),
        },
    })
    .unwrap()
}
pub fn unwrap(message: &[u8]) -> Result<(Vec<u8>, Metadata), EnvelopeError> {
    let envelope: Envelope = serde_json::from_slice(message).map_err(EnvelopeError::JsonError)?;

    Ok((
        general_purpose::URL_SAFE_NO_PAD
            .decode(envelope.content)
            .map_err(EnvelopeError::Base64Error)?,
        envelope.metadata,
    ))
}
