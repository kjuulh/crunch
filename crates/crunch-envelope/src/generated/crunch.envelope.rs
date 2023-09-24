#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Envelope {
    #[prost(message, optional, tag = "1")]
    pub metadata: ::core::option::Option<Metadata>,
    #[prost(bytes = "vec", tag = "2")]
    pub content: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Metadata {
    #[prost(string, tag = "1")]
    pub domain: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub entity: ::prost::alloc::string::String,
    #[prost(uint64, tag = "3")]
    pub timestamp: u64,
    #[prost(uint64, tag = "4")]
    pub sequence: u64,
}
