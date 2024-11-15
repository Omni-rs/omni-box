use alloy::node_bindings::AnvilInstance;
use alloy::{
    network::EthereumWallet, node_bindings::Anvil, providers::ProviderBuilder,
    signers::local::PrivateKeySigner,
};

pub fn get_anvil_instance() -> Result<AnvilInstance, Box<dyn std::error::Error>> {
    // Spin up a local Anvil node.
    let anvil = Anvil::new().block_time(1).try_spawn()?;

    // Configure the signer from the first default Anvil account (Alice).
    let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
    let wallet = EthereumWallet::from(signer.clone());

    // Create a provider with the wallet.
    let rpc_url = anvil.endpoint().parse()?;
    let _provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet.clone())
        .on_http(rpc_url);

    Ok(anvil)
}
