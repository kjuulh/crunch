#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Envelope {
    #[prost(message, optional, tag="1")]
    pub metadata: ::std::option::Option<Metadata>,
    #[prost(bytes, tag="2")]
    pub content: std::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Metadata {
    #[prost(string, tag="1")]
    pub domain: std::string::String,
    #[prost(string, tag="2")]
    pub entity: std::string::String,
    #[prost(uint64, tag="3")]
    pub timestamp: u64,
    #[prost(uint64, tag="4")]
    pub sequence: u64,
}
