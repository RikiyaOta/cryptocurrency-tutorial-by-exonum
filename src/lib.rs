pub mod proto;

/// Persistent data.
pub mod schema {
    use exonum::crypto::PublicKey;
    use exonum_derive::{BinaryValue, ObjectHash};
    use exonum_proto::ProtobufConvert;
    use serde::{Deserialize, Serialize};

    use crate::proto;

    // Persistent Data structures.

    #[derive(Clone, Debug)]
    #[derive(Serialize, Deserialize)]
    #[derive(ProtobufConvert, BinaryValue, ObjectHash)]
    #[protobuf_convert(source = "proto::Wallet")]
    pub struct Wallet {
        pub pub_key: PublicKey,
        pub name: String,
        pub balance: u64,
    }
}