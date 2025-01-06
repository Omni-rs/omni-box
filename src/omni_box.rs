use crate::{
    account_config::get_user_account_info_from_file,
    chain_config::ChainConfig,
    contexts::{BTCTestContext, EVMTestContext, NearTestContext},
    friendly_near_json_rpc_client::FriendlyNearJsonRpcClient,
    network::Network,
    omni_box_options::OmniBoxOptions,
    utils::address,
    NearAccount,
};
use alloy::{
    hex::FromHex,
    primitives::{utils::parse_units, Address, U256},
    providers::ext::AnvilApi,
};
use serde_json::json;
use sha3::{Digest, Sha3_256};
use std::fs::{self, OpenOptions};
use std::io::prelude::*;
use std::path::Path;
use std::{collections::HashMap, error::Error};

pub struct OmniBox {
    chains: HashMap<Network, ChainConfig>,
    pub btc_context: BTCTestContext,
    pub near_context: NearTestContext,
    pub evm_context: EVMTestContext,
    pub deployer_account: NearAccount,
    pub friendly_near_json_rpc_client: FriendlyNearJsonRpcClient,
}

const MPC_SIGNER: &str = "v1.signer-prod.testnet";

impl OmniBox {
    pub async fn new() -> Self {
        Self::new_with_conf(None).await
    }

    pub async fn new_with_conf(options: Option<OmniBoxOptions>) -> Self {
        let mut chains = HashMap::new();
        let options = options.unwrap_or_default();

        println!("Starting OmniBox with options: {:#?}", options);

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
        let friendly_client =
            FriendlyNearJsonRpcClient::new(options.default_near_network, deployer_account.clone());
        println!("Using deployer account: {:#?}", deployer_account.account_id);

        // Create the OmniBox instance, each context will be initialized with the default configuration
        let omnibox = Self {
            chains,
            btc_context: BTCTestContext::default(),
            near_context: NearTestContext::new().await,
            evm_context: EVMTestContext::default(),
            deployer_account: deployer_account.clone(),
            friendly_near_json_rpc_client: friendly_client,
        };

        // Calculate derived addresses for Bitcoin legacy
        let legacy_derived_address = address::get_derived_address_for_btc_legacy(
            &deployer_account.account_id,
            options.btc_path,
        );

        println!(
            "Legacy BTC Derived Address: {:?}",
            legacy_derived_address.address
        );

        // Calculate derived addresses for Bitcoin Segwit
        let segwit_derived_address =
            address::get_derived_address_for_segwit(&deployer_account.account_id, options.btc_path);

        println!(
            "Segwit BTC Derived Address: {:?}",
            segwit_derived_address.address
        );

        // Calculate default derived addresses EVM
        let evm_derived_address =
            address::get_derived_address_for_evm(&deployer_account.account_id, &options.evm_path);

        println!("EVM Derived Address: {:?}", evm_derived_address.address);

        // Give initial funds to the deployer account in EVM
        let mocked_balance: U256 = match parse_units("100.0", "ether") {
            Ok(units) => units.into(),
            Err(e) => {
                eprintln!("Failed to parse units: {}", e);
                return omnibox;
            }
        };

        omnibox
            .evm_context
            .provider
            .anvil_set_balance(
                Address::from_hex(evm_derived_address.address).unwrap(),
                mocked_balance,
            )
            .await
            .unwrap();

        // Auto compile and deploy
        omnibox
            .compile_and_deploy_contract(options.path)
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
    ) -> Result<(), Box<dyn std::error::Error>> {
        let cache_path = "cache/contract.json";

        // Compile the contract
        println!("Compiling contract");

        let contract_wasm = near_workspaces::compile_project(path).await?;

        // Calculate the current hash of the contract
        let current_hash = self.calculate_hash_from_bytes(&contract_wasm);

        // Read the cached hash
        let cached_hash = self.read_hash_from_cache(cache_path).unwrap_or_default();

        // Check if the hash has changed
        if current_hash == cached_hash {
            println!("Contract has not changed, skipping deployment.");
            return Ok(());
        }

        // Deploy the contract
        self.friendly_near_json_rpc_client
            .deploy_contract(contract_wasm)
            .await?;

        // Update the cache with the new hash
        self.write_hash_to_cache(cache_path, &current_hash)?;

        println!("Contract deployed");

        Ok(())
    }

    pub async fn get_experimental_signature_deposit(&self) -> Result<u128, Box<dyn Error>> {
        let method_name = "experimental_signature_deposit";
        let args = json!({});

        self.friendly_near_json_rpc_client
            .call_contract_with_account_id::<u128>(MPC_SIGNER, method_name, args)
            .await
    }

    // Caching capabilities
    fn calculate_hash_from_bytes(&self, bytes: &[u8]) -> String {
        let mut hasher = Sha3_256::new();
        hasher.update(bytes);
        let hash = hasher.finalize();
        format!("{:x}", hash)
    }

    fn write_hash_to_cache<P: AsRef<Path>>(
        &self,
        cache_path: P,
        hash: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let cache_path = cache_path.as_ref();
        if let Some(parent) = cache_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(cache_path)?;
        writeln!(file, "{}", hash)?;
        Ok(())
    }

    fn read_hash_from_cache<P: AsRef<Path>>(
        &self,
        cache_path: P,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let hash = std::fs::read_to_string(cache_path)?;
        Ok(hash.trim().to_string())
    }
}
