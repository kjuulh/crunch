fn main() {
    #[no_mangle]
    fn envelope_capnp_benchmark(content: &[u8]) {
        let out = crunch_envelope::wrap("some-domain", "some-entity", content);

        let out = crunch_envelope::unwrap(&out).expect("to be able to unwrap capnp message");

        println!("{:?}", out.1);
    }

    #[no_mangle]
    fn envelope_json_benchmark(content: &[u8]) {
        let out = crunch_envelope::json::wrap("some-domain", "some-entity", content);

        let out = crunch_envelope::json::unwrap(&out).expect("to be able to unwrap capnp message");

        println!("{:?}", out.1);
    }

    let large_content: [u8; 1000000] = [0; 1000000];

    envelope_capnp_benchmark(&large_content);;

    envelope_json_benchmark(&large_content);;

    println!("done")
}
