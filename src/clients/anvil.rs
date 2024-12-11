use alloy::node_bindings::Anvil;
use alloy::node_bindings::AnvilInstance;

pub fn get_anvil_instance() -> Result<AnvilInstance, Box<dyn std::error::Error>> {
    // Spin up a local Anvil node.
    let anvil = Anvil::new().block_time(1).try_spawn()?;

    Ok(anvil)
}
