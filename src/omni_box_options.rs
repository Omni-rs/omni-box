use crate::{chain_config::ChainOverrides, network::Network};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct OmniBoxOptions {
    pub modules: Vec<Network>,                       // Networks to include
    pub overrides: HashMap<Network, ChainOverrides>, // Overrides for each network
}

impl Default for OmniBoxOptions {
    fn default() -> Self {
        OmniBoxOptions {
            modules: vec![Network::Ethereum, Network::Near, Network::Bitcoin],
            overrides: HashMap::new(),
        }
    }
}
