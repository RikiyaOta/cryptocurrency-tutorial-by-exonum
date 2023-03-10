use exonum_build::ProtobufGenerator;

fn main() {
    ProtobufGenerator::with_mod_name("protobuf_mod.rs")
        .with_input_dir("src/proto")
        .with_crypto()
        //.with_merkledb()
        //.with_exonum()
        //.with_common()
        .generate();
}
