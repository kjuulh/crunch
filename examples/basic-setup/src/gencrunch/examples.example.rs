#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MyEvent {
    #[prost(string, tag = "1")]
    pub my_field: ::prost::alloc::string::String,
}
