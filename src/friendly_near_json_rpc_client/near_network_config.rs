//! Define the network configuration for the OmniBox environment.
/// Define the network configuration for the OmniBox environment.
#[derive(Debug, Clone, Copy)]
pub enum NearNetworkConfig {
    Testnet,
    Mainnet,
    Local,
}

/// Get the RPC URL for the given network configuration.
pub const fn get_rpc_url(network: NearNetworkConfig) -> &'static str {
    match network {
        NearNetworkConfig::Testnet => "https://rpc.testnet.near.org",
        NearNetworkConfig::Mainnet => "https://rpc.mainnet.near.org",
        NearNetworkConfig::Local => "http://localhost:3030",
    }
}
