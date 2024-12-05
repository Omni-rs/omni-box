use alloy::node_bindings::AnvilInstance;
use alloy::{
    network::EthereumWallet, node_bindings::Anvil, providers::ProviderBuilder,
    signers::local::PrivateKeySigner,
};

pub fn get_anvil_instance() -> Result<AnvilInstance, Box<dyn std::error::Error>> {
    // Spin up a local Anvil node.
    let anvil = Anvil::new().block_time(1).try_spawn()?;

    Ok(anvil)
}
