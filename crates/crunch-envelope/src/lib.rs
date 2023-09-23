mod envelope_capnp;

#[cfg(feature = "json")]
mod json_envelope;

mod generated;
#[cfg(feature = "proto")]
mod proto_envelope;

#[cfg(feature = "json")]
pub mod json {
    pub use crate::json_envelope::*;
}

#[cfg(feature = "proto")]
pub mod proto {
    pub use crate::proto_envelope::*;
}

use capnp::message::{Builder, ReaderOptions};
use capnp::serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EnvelopeError {
    #[error("capnp failed to serialize or deserialize code")]
    CapnpError(#[source] capnp::Error),
    #[cfg(feature = "json")]
    #[error("serde_json failed to serialize or deserialize code")]
    JsonError(#[source] serde_json::Error),
    #[cfg(feature = "json")]
    #[error("base64 failed to serialize or deserialize code")]
    Base64Error(#[source] base64::DecodeError),
    #[cfg(feature = "proto")]
    #[error("prost failed to serialize or deserialize code")]
    ProtoError(#[source] prost::DecodeError),
    #[error("metadata is missing from field")]
    MetadataError(),
}

pub fn wrap<'a>(domain: &'a str, entity: &'a str, content: &'a [u8]) -> Vec<u8> {
    let mut builder = Builder::new_default();
    let mut envelope = builder.init_root::<envelope_capnp::envelope::Builder>();
    envelope.set_content(content);

    let mut metadata = envelope.init_metadata();
    metadata.set_domain(domain);
    metadata.set_entity(entity);

    let output = serialize::write_message_to_words(&builder);

    return output;
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Metadata {
    domain: String,
    entity: String,
}

pub fn unwrap<'a>(message: &'a [u8]) -> Result<(Vec<u8>, Metadata), EnvelopeError> {
    let mut message = message;
    let message_builder =
        serialize::read_message_from_flat_slice(&mut message, ReaderOptions::new())
            .map_err(EnvelopeError::CapnpError)?;

    let envelope = message_builder
        .get_root::<envelope_capnp::envelope::Reader>()
        .map_err(EnvelopeError::CapnpError)?;
    let content = envelope.get_content().map_err(EnvelopeError::CapnpError)?;
    let metadata = envelope.get_metadata().map_err(EnvelopeError::CapnpError)?;

    Ok((
        content.to_vec(),
        Metadata {
            domain: metadata
                .get_domain()
                .map_err(EnvelopeError::CapnpError)?
                .to_string(),
            entity: metadata
                .get_entity()
                .map_err(EnvelopeError::CapnpError)?
                .to_string(),
        },
    ))
}

#[cfg(test)]
mod test {
    use crate::{unwrap, wrap};

    #[test]
    fn test_can_serialize() {
        let domain = "some-domain";
        let entity = "some-entity";
        let message = b"some-content";
        let out = wrap(domain, entity, message);

        let original = unwrap(&out).expect("to be able to unwrap capnp message");

        assert_eq!(domain, original.1.domain);
        assert_eq!(entity, original.1.entity);
        assert_eq!(message, original.0.as_slice());
    }
}
