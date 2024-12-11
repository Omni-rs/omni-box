#[derive(Debug, Clone, Copy)]
pub enum NearNetworkConfig {
    Testnet,
    Mainnet,
    Local,
}

pub const fn get_rpc_url(network: NearNetworkConfig) -> &'static str {
    match network {
        NearNetworkConfig::Testnet => "https://rpc.testnet.near.org",
        NearNetworkConfig::Mainnet => "https://rpc.mainnet.near.org",
        NearNetworkConfig::Local => "http://localhost:3030",
    }
}
