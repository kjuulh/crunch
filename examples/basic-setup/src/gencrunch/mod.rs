pub mod basic {
    pub mod includes {
        pub mod my_include {
            use prost::Message;
            include!("basic.includes.my_include.rs");
            impl ::crunch::traits::Serializer for MyInclude {
                fn serialize(&self) -> Result<Vec<u8>, ::crunch::errors::SerializeError> {
                    Ok(self.encode_to_vec())
                }
            }
            impl ::crunch::traits::Deserializer for MyInclude {
                fn deserialize(raw: Vec<u8>) -> Result<Self, ::crunch::errors::DeserializeError>
                where
                    Self: Sized,
                {
                    let output = Self::decode(raw.as_slice())
                        .map_err(|e| ::crunch::errors::DeserializeError::ProtoErr(e))?;
                    Ok(output)
                }
            }
            impl crunch::traits::Event for MyInclude {
                fn event_info() -> ::crunch::traits::EventInfo {
                    ::crunch::traits::EventInfo {
                        domain: "my-domain".into(),
                        entity_type: "my-entity-type".into(),
                        event_name: "my-event-name".into(),
                    }
                }
            }
        }
    }
    pub mod my_event {
        use prost::Message;
        include!("basic.my_event.rs");
        impl ::crunch::traits::Serializer for MyEvent {
            fn serialize(&self) -> Result<Vec<u8>, ::crunch::errors::SerializeError> {
                Ok(self.encode_to_vec())
            }
        }
        impl ::crunch::traits::Deserializer for MyEvent {
            fn deserialize(raw: Vec<u8>) -> Result<Self, ::crunch::errors::DeserializeError>
            where
                Self: Sized,
            {
                let output = Self::decode(raw.as_slice())
                    .map_err(|e| ::crunch::errors::DeserializeError::ProtoErr(e))?;
                Ok(output)
            }
        }
        impl crunch::traits::Event for MyEvent {
            fn event_info() -> ::crunch::traits::EventInfo {
                ::crunch::traits::EventInfo {
                    domain: "my-domain".into(),
                    entity_type: "my-entity-type".into(),
                    event_name: "my-event-name".into(),
                }
            }
        }
    }
}
pub mod examples {
    pub mod example {
        use prost::Message;
        include!("examples.example.rs");
        impl ::crunch::traits::Serializer for MyEvent {
            fn serialize(&self) -> Result<Vec<u8>, ::crunch::errors::SerializeError> {
                Ok(self.encode_to_vec())
            }
        }
        impl ::crunch::traits::Deserializer for MyEvent {
            fn deserialize(raw: Vec<u8>) -> Result<Self, ::crunch::errors::DeserializeError>
            where
                Self: Sized,
            {
                let output = Self::decode(raw.as_slice())
                    .map_err(|e| ::crunch::errors::DeserializeError::ProtoErr(e))?;
                Ok(output)
            }
        }
        impl crunch::traits::Event for MyEvent {
            fn event_info() -> ::crunch::traits::EventInfo {
                ::crunch::traits::EventInfo {
                    domain: "my-domain".into(),
                    entity_type: "my-entity-type".into(),
                    event_name: "my-event-name".into(),
                }
            }
        }
    }
}
