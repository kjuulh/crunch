pub mod envelope {
    include!(concat!(env!("OUT_DIR"), "/crunch.envelope.rs"));
}

use prost::Message;

use crate::EnvelopeError;

pub fn wrap<'a>(domain: &'a str, entity: &'a str, content: &'a [u8]) -> Vec<u8> {
    let out = envelope::Envelope {
        metadata: Some(envelope::Metadata {
            domain: domain.to_string(),
            entity: entity.to_string(),
            timestamp: 0,
            sequence: 0,
        }),
        content: content.to_vec(),
    };

    out.encode_to_vec()
}

pub fn unwrap<'a>(message: &'a [u8]) -> Result<(Vec<u8>, envelope::Metadata), EnvelopeError> {
    let out = envelope::Envelope::decode(message).map_err(EnvelopeError::ProtoError)?;

    Ok((
        out.content,
        out.metadata.ok_or(EnvelopeError::MetadataError())?,
    ))
}
