#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MyInclude {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
}
