use exonum_cli::{NodeBuilder, Spec};

use cryptocurrency_tutorial_by_exonum::contracts::CryptocurrencyService;

#[tokio::main(basic_scheduler)]
async fn main() -> anyhow::Result<()> {
    exonum::helpers::init_logger()?;

    NodeBuilder::development_node()?
        .with(Spec::new(CryptocurrencyService).with_default_instance())
        .run()
        .await
}