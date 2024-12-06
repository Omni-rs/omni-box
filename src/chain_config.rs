use crate::network::Network;
use crate::Account;
use alloy::node_bindings::AnvilInstance;
use bitcoind::BitcoinD;
use near_workspaces::network::Sandbox;
use near_workspaces::Worker;

#[derive(Debug)]
pub enum NodeInstance {
    Anvil(AnvilInstance),
    Bitcoin(BitcoinD),
    Workspaces(Worker<Sandbox>),
}

#[derive(Debug)]
pub struct ChainConfig {
    pub node_url: String,
    pub node_instance: Option<NodeInstance>,
    pub accounts: Vec<Option<Account>>,
}

#[derive(Debug, Clone)]
pub struct ChainOverrides {
    pub node_url: Option<String>,
}

impl ChainConfig {
    pub fn default(network: Network) -> Self {
        match network {
            Network::Ethereum => Self {
                node_url: "http://localhost:8545".to_string(),
                node_instance: None,
                accounts: vec![None],
            },
            Network::Near => Self {
                node_url: "https://localhost:3030".to_string(),
                node_instance: None,
                accounts: vec![None],
            },
            Network::Bitcoin => Self {
                node_url: "http://localhost:18443".to_string(),
                node_instance: None,
                accounts: vec![None],
            },
        }
    }
}
