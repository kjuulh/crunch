#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MyEvent {
    #[prost(string, tag="1")]
    pub name: std::string::String,
    #[prost(message, optional, tag="2")]
    pub include: ::std::option::Option<super::includes::my_include::MyInclude>,
}
