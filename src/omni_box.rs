use crate::{
    account_config::get_user_account_info_from_file,
    chain_config::ChainConfig,
    contexts::{BTCTestContext, EVMTestContext, NearTestContext},
    friendly_near_json_rpc_client::{
        near_network_config::NearNetworkConfig, FriendlyNearJsonRpcClient,
    },
    network::Network,
    omni_box_options::OmniBoxOptions,
    NearAccount,
};
use std::collections::HashMap;

pub struct OmniBox {
    chains: HashMap<Network, ChainConfig>,
    pub btc_context: BTCTestContext,
    pub near_context: NearTestContext,
    pub evm_context: EVMTestContext,
    pub deployer_account: NearAccount,
}

impl OmniBox {
    pub async fn new() -> Self {
        Self::new_with_conf(None).await
    }

    pub async fn new_with_conf(options: Option<OmniBoxOptions>) -> Self {
        let mut chains = HashMap::new();
        let options = options.unwrap_or_default();

        println!("Creating OmniBox with options: {:#?}", options);

        for module in options.modules {
            // Create a default configuration for this module / chain / network
            let mut config = ChainConfig::default(module.clone());

            // Apply overrides if they exist
            if let Some(overrides) = options.overrides.get(&module) {
                if let Some(url) = &overrides.node_url {
                    config.node_url = url.clone();
                }
            }
            chains.insert(module, config);
        }

        // Get the deployer account
        let deployer_account = get_user_account_info_from_file(None).unwrap();

        // Create the OmniBox instance, each context will be initialized with the default configuration
        let omnibox = Self {
            chains,
            btc_context: BTCTestContext::default(),
            near_context: NearTestContext::new().await,
            evm_context: EVMTestContext::default(),
            deployer_account,
        };

        // Auto compile and deploy
        omnibox
            .compile_and_deploy_contract(options.path, options.default_near_network)
            .await
            .unwrap();

        omnibox
    }

    pub fn get_chain_config(&self, network: &Network) -> Option<&ChainConfig> {
        self.chains.get(network)
    }

    // Near utils
    async fn compile_and_deploy_contract(
        &self,
        path: &'static str,
        network: NearNetworkConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Compile the contract
        println!("Compiling contract");

        let contract_wasm = near_workspaces::compile_project(path).await?;

        let config_account = get_user_account_info_from_file(None).unwrap();

        let friendly_client = FriendlyNearJsonRpcClient::new(network, config_account);

        // Deploy the contract
        friendly_client.deploy_contract(contract_wasm).await?;

        println!("Contract deployed");

        Ok(())
    }
}
