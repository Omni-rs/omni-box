use crate::{chain_config::ChainOverrides, network::Network, NearNetworkConfig};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct OmniBoxOptions {
    pub modules: Vec<Network>,                       // Networks to include
    pub overrides: HashMap<Network, ChainOverrides>, // Overrides for each network
    pub path: &'static str,                          // Path to the config file
    pub default_near_network: NearNetworkConfig,     // Default Near network
}

impl Default for OmniBoxOptions {
    fn default() -> Self {
        Self {
            modules: vec![Network::Ethereum, Network::Near, Network::Bitcoin],
            overrides: HashMap::new(),
            path: "./",
            default_near_network: NearNetworkConfig::Testnet,
        }
    }
}
