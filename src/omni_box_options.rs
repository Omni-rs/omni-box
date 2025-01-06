use crate::{chain_config::ChainOverrides, network::Network, NearNetworkConfig};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct OmniBoxOptions {
    pub modules: Vec<Network>,                       // Networks to include
    pub overrides: HashMap<Network, ChainOverrides>, // Overrides for each network
    pub path: &'static str,                          // Path to the config file
    pub default_near_network: NearNetworkConfig,     // Default Near network
    pub btc_path: &'static str,                      // Default path of the Bitcoin address
    pub evm_path: &'static str,                      // Default path of the EVM address
}

const DEFAULT_BTC_PATH: &str = "bitcoin-1";
const DEFAULT_EVM_PATH: &str = "ethereum-1";

impl Default for OmniBoxOptions {
    fn default() -> Self {
        Self {
            modules: vec![Network::EVM, Network::Near, Network::Bitcoin],
            overrides: HashMap::new(),
            path: "./",
            default_near_network: NearNetworkConfig::Testnet,
            btc_path: DEFAULT_BTC_PATH,
            evm_path: DEFAULT_EVM_PATH,
        }
    }
}
