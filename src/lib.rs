use exonum::{
    blockchain::Schema,
    merkledb::{access::Access, MapIndex},
    proto::schema::{auth, errors::ExecutionError},
};
use exonum_crypto::PublicKey;
use exonum_derive::{exonum_interface, interface_method, FromAccess};
use exonum_rust_runtime::{
    api::{self, ServiceApiBuilder, ServiceApiState},
    DefaultInstance, ExecutionContext,
};

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

/// Cryptocurrency service implementation.
#[drive(Debug, ServiceFactory, ServiceDispatcher)]
#[service_dispatcher(implements("CryptocurrencyInterface"))]
#[service_factory(proto_sources = "crate::proto")]
pub struct CryptocurrencyService;

impl Service for CryptocurrencyService {
    fn wire_api(&self, builder: &mut ServiceApiBuilder) {
        CryptocurrencyApi::wire(builder)
    }
}

impl DefaultInstance for CryptocurrencyService {
    const INSTANCE_ID: u32 = 101;
    const INSTANCE_NAME: &'static str = "cryptocurrency";
}

impl CryptocurrencyInterface<ExecutionContext<'_>> for CryptocurrencyService {
    type Output = Result<(), ExecutionError>;

    fn create_wallet(&self, ctx: ExecutionContext<'_>, arg: TxCreateWallet) -> Self::Output {
        // Warning:
        // Calling `expect` is not suitable for production use. Consider `CallerAddress`.
        let author = ctx
            .caller()
            .author()
            .expect("Wrong 'TxCreateWallet' initiator");
        let mut schema = CurrencySchema::new(ctx.service_data());

        if schema.wallets.get(&author).is_none() {
            let wallet = Wallet::new(&author, &arg.name, INIT_BALANCE);
            println!("Created wallet: {:?}", wallet);
            schema.wallets.put(&author, wallet);
            Ok(())
        } else {
            Err(Error::WalletAlreadyExists.into())
        }
    }

    fn transfer(&self, ctx: ExecutionContext<'_>, arg: TxTransfer) -> Self::Output {
        let author = ctx.caller().author().expect("Wrong 'TxTransfer' initiator");

        if author == arg.to {
            return Err(Error::SenderSameAsReceiver.info());
        }

        let mut schema = CurrencySchema::new(ctx.service_data());
        let sender = schema.wallets.get(&author).ok_or(Error::SenderNotFound)?;
        let receiver = schema.wallets.get(&arg.to).ok_or(Error::ReceiverNotFound)?;
        let amount = arg.amount;

        if sender.balance >= amount {
            let sender = sender.decrease(amount);
            let receiver = receiver.increase(amount);
            println!("Transfer between wallets: {:?} => {:?}", sender, receiver);
            schema.wallets.put(&author, sender);
            schema.wallets.put(&arg.to, receiver);
            Ok(())
        } else {
            Err(Error::InsufficientCurrencyAmount.into())
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct CryptocurrencyApi;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct WalletQuery {
    /// Public key of the requested wallet.
    pub pub_key: PublicKey,
}

type Handle<Query, Response> = fn(&ServiceApiState<'_>, Query) -> api::Result<Response>;

impl CryptocurrencyApi {
    /// Endpoint for getting a single wallet.
    pub fn get_wallet(state: &ServiceApiState<'_>, pub_key: PublicKey) -> api::Result<Wallet> {
        let schema = CurrencySchema::new(state.service_data());
        schema
            .wallets
            .get(&pub_key)
            .ok_or_else(|| api::Error::not_found().title("Wallet not found"))
    }

    /// Endpoint for dumping all wallets from the storage.
    pub fn get_wallets(state: &ServiceApiState<'_>, _query: ()) -> api::Result<Vec<Wallet>> {
        let schema = CurrencySchema::new(state.service_data());
        Ok(schema.wallets.values().collect())
    }

    /// `ServiceApiBuilder` facilitates conversation between read requests
    /// and REST endpoints.
    pub fn wire(builder: &mut ServiceApiBuilder) {
        bulder
            .public_scope()
            .endpoint("v1/wallet", Self::get_wallet)
            .endpoint("v1/wallets", Self::get_wallets);
    }
}
