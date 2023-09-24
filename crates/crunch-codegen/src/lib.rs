#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_can_generate_output_rust() {
        let proto_spec = r#"
syntax = "proto3";

package test.can.generate.output.rust;

message MyEvent {
    string name = 1;
}
"#;
    }
}
