extern crate capnpc;

fn main() {
    capnpc::CompilerCommand::new()
        .output_path("src/")
        .src_prefix("schemas/")
        .file("schemas/envelope.capnp")
        .run()
        .unwrap();

    std::fs::create_dir_all("src/generated").unwrap();
    let mut config = prost_build::Config::default();
    config.out_dir("src/generated/");

    config
        .compile_protos(&["src/envelope.proto"], &["src/"])
        .unwrap();
}
