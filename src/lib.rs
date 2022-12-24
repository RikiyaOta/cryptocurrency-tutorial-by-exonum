use exonum::merkledb::access::Access;
use exonum_crypto::PublicKey;
use exonum_derive::{exonum_interface, interface_method};

mod proto;

const INIT_BALANCE: u64 = 100;

#[derive(Serialize, Deserialize, Clone, Debug, ProtobufConvert, BinaryValue, ObjectHash)]
#[protobuf_convert(source = "proto::Wallet")]
pub struct Wallet {
    pub pub_key: PublicKey,
    pub name: String,
    pub balance: u64,
}

impl Wallet {
    pub fn new(&pub_key: &PublicKey, name: &str, balance: u64) -> Self {
        Self {
            pub_key,
            name: name.to_owned(),
            balance,
        }
    }

    pub fn increase(self, amount: u64) -> Self {
        let balance = self.balance + amount;
        Self::new(&self.pub_key, &self.name, balance)
    }

    pub fn decrease(self, amount: u64) -> Self {
        debug_assert!(self.balance >= amount);
        let balance = self.balance - amount;
        Self::new(&self.pub_key, &self.name, balance)
    }
}

#[derive(Debug, FromAccess)]
pub struct CurrencySchema<T: Access> {
    pub wallets: MapIndex<T::Base, PublicKey, Wallet>,
}

impl<T: Access> CurrencySchema<T> {
    pub fn new(access: T) -> Self {
        Self::from_root(access).unwrap()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, ProtobufConvert, BinaryValue)]
#[protobuf_convert(source = "proto::TxCreateWallet")]
pub struct TxCreateWallet {
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, ProtobufConvert, BinaryValue)]
#[protobuf_convert(source = "proto::TxTransfer")]
pub struct TxTransfer {
    pub to: PublicKey,
    pub amount: u64,
    pub seed: u64,
}

/// Cryptocurrency service transactions.
#[exonum_interface]
pub trait CryptocurrencyInterface<Ctx> {
    /// Output of the methods in this interface.
    type Output;

    /// Creates wallet with the given `name`.
    #[interface_method(id = 0)]
    fn create_wallet(&self, ctx: Ctx, arg: TxCreateWallet) -> Self::Output;

    #[interface_method(id = 1)]
    fn transfer(&self, ctx: Ctx, arg: TxTransfer) -> Self::Output;
}

/// Error codes emitted by `TxCreateWallet` and/or `TxTransfer`
/// transactions during execution.
#[derive(Debug, ExecutionFail)]
pub enum Error {
    WalletAlreadyExists = 0,
    SenderNotFound = 1,
    ReceiverNotFound = 2,
    InsufficientCurrencyAmount = 3,
    SenderSameAsReceiver = 4,
}
