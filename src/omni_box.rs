use crate::{chain_config::ChainConfig, network::Network, omni_box_options::OmniBoxOptions};
use std::collections::HashMap;

pub struct OmniBox {
    chains: HashMap<Network, ChainConfig>,
}

impl OmniBox {
    pub fn new() -> Self {
        Self::new_with_conf(None)
    }

    pub fn new_with_conf(options: Option<OmniBoxOptions>) -> Self {
        let mut chains = HashMap::new();
        let options = options.unwrap_or_default();

        for module in options.modules {
            // Create a default configuration for this module
            let mut config = ChainConfig::default(module.clone());

            // Apply overrides if they exist
            if let Some(overrides) = options.overrides.get(&module) {
                if let Some(url) = &overrides.node_url {
                    config.node_url = url.clone();
                }
                if let Some(accounts) = &overrides.default_accounts {
                    config.default_accounts = accounts.clone();
                }
            }

            chains.insert(module, config);
        }
        // Create the OmniBox instance and initialize the accounts
        let omni_box = OmniBox { chains };
        omni_box.initialize_accounts();
        omni_box
    }

    fn initialize_accounts(&self) {
        for (name, config) in &self.chains {
            println!(
                "Initializing {:?} accounts at {}: {:?}",
                name, config.node_url, config.default_accounts
            );
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChainOverrides {
    pub node_url: Option<String>,
    pub default_accounts: Option<Vec<String>>,
}
