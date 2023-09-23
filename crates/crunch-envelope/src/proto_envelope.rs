use prost::Message;

use crate::generated::crunch::*;
use crate::EnvelopeError;

pub fn wrap<'a>(domain: &'a str, entity: &'a str, content: &'a [u8]) -> Vec<u8> {
    let out = Envelope {
        metadata: Some(Metadata {
            domain: domain.to_string(),
            entity: entity.to_string(),
            timestamp: 0,
            sequence: 0,
        }),
        content: content.to_vec(),
    };

    out.encode_to_vec()
}

pub fn unwrap<'a>(message: &'a [u8]) -> Result<(Vec<u8>, Metadata), EnvelopeError> {
    let out = Envelope::decode(message).map_err(EnvelopeError::ProtoError)?;

    Ok((
        out.content,
        out.metadata.ok_or(EnvelopeError::MetadataError())?,
    ))
}
