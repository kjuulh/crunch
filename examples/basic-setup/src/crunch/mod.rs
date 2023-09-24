pub mod basic {
    pub mod my_event {
    include!("basic.my_event.rs");
    
    impl ::crunch::Serializer for MyEvent {
     fn serialize(&self) -> Result<Vec<u8>, ::crunch::errors::SerializeError> {
     todo!()
     }
    }
    impl ::crunch::Deserializer for MyEvent {
     fn deserialize(_raw: Vec<u8>) -> Result<Self, ::crunch::errors::DeserializeError>
     where
     Self: Sized,
     {
     todo!()
     }
    }
    
    impl Event for MyEvent {
     fn event_info() -> ::crunch::traits::EventInfo {
     EventInfo {
     domain: "my-domain",
     entity_type: "my-entity-type",
     event_name: "my-event-name",
     }
     }
    }
    }
    pub mod includes {     
        pub mod my_include {
        include!("basic.includes.my_include.rs");
        
        impl ::crunch::Serializer for MyInclude {
         fn serialize(&self) -> Result<Vec<u8>, ::crunch::errors::SerializeError> {
         todo!()
         }
        }
        impl ::crunch::Deserializer for MyInclude {
         fn deserialize(_raw: Vec<u8>) -> Result<Self, ::crunch::errors::DeserializeError>
         where
         Self: Sized,
         {
         todo!()
         }
        }
        
        impl Event for MyInclude {
         fn event_info() -> ::crunch::traits::EventInfo {
         EventInfo {
         domain: "my-domain",
         entity_type: "my-entity-type",
         event_name: "my-event-name",
         }
         }
        }
        }
    }
}
