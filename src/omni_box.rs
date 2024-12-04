use crate::{
    account_config::get_user_account_info_from_file,
    chain_config::{ChainConfig, NodeInstance},
    friendly_near_json_rpc_client::{
        near_network_config::NearNetworkConfig, FriendlyNearJsonRpcClient,
    },
    network::Network,
    nodes::{get_anvil_instance, get_bitcoin_instance, get_near_instance},
    omni_box_options::OmniBoxOptions,
};
use std::collections::HashMap;

pub struct OmniBox {
    pub chains: HashMap<Network, ChainConfig>,
}

impl OmniBox {
    pub async fn new() -> Self {
        Self::new_with_conf(None).await
    }

    pub async fn new_with_conf(options: Option<OmniBoxOptions>) -> Self {
        let mut chains = HashMap::new();
        let options = options.unwrap_or_default();

        for module in options.modules {
            // Create a default configuration for this module (o sea network)
            let mut config = ChainConfig::default(module.clone());

            // Apply overrides if they exist
            if let Some(overrides) = options.overrides.get(&module) {
                if let Some(url) = &overrides.node_url {
                    config.node_url = url.clone();
                }
            }

            chains.insert(module, config);
        }

        // Create the OmniBox instance and initialize the accounts and nodes
        let mut omni_box = OmniBox { chains };
        omni_box.initialize_nodes().await;
        omni_box.initialize_accounts();
        omni_box
    }

    // Internal methods
    async fn initialize_nodes(&mut self) {
        for (network, config) in &mut self.chains {
            match network {
                Network::Ethereum => {
                    if let Ok(anvil) = get_anvil_instance() {
                        config.node_instance = Some(NodeInstance::Anvil(anvil));
                        println!("Initialized Ethereum node at {}", config.node_url);
                    }
                }
                Network::Bitcoin => {
                    if let Ok(bitcoind) = get_bitcoin_instance() {
                        config.node_instance = Some(NodeInstance::Bitcoin(bitcoind));
                        println!("Initialized Bitcoin node at {}", config.node_url);
                    }
                }
                Network::Near => {
                    if let Ok(near) = get_near_instance().await {
                        config.node_instance = Some(NodeInstance::Workspaces(near));
                        println!("Initialized Near node at {}", config.node_url);
                    }
                }
            }
        }
    }

    fn initialize_accounts(&mut self) {
        for (network, config) in &mut self.chains {
            match network {
                Network::Near => {
                    let account_info = get_user_account_info_from_file(None).unwrap();
                    config.accounts = vec![Some(account_info.into())];
                }
                _ => {}
            }
        }
    }

    // Near utils
    pub async fn compile_and_deploy_contract(
        &self,
        path: &str,
        network: NearNetworkConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Compile the contract
        let contract_wasm = near_workspaces::compile_project(path).await?;

        let config_account = get_user_account_info_from_file(None).unwrap();

        let friendly_client = FriendlyNearJsonRpcClient::new(network, config_account);

        // Deploy the contract
        friendly_client.deploy_contract(contract_wasm).await?;

        println!("Contract deployed");

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ChainOverrides {
    pub node_url: Option<String>,
}
