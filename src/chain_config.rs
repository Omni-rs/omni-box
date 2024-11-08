use crate::network::Network;

#[derive(Debug, Clone)]
pub struct ChainConfig {
    pub node_url: String,
    pub default_accounts: Vec<String>,
}

impl ChainConfig {
    pub fn default(network: Network) -> Self {
        match network {
            Network::Ethereum => ChainConfig {
                node_url: "http://localhost:8545".to_string(),
                default_accounts: vec!["alice".to_string(), "bob".to_string()],
            },
            Network::Near => ChainConfig {
                node_url: "https://rpc.testnet.near.org".to_string(),
                default_accounts: vec!["alice.testnet".to_string(), "bob.testnet".to_string()],
            },
            Network::Bitcoin => ChainConfig {
                node_url: "http://localhost:18443".to_string(),
                default_accounts: vec!["alice_btc".to_string(), "bob_btc".to_string()],
            },
        }
    }
}
