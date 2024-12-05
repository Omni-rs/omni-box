use near_crypto::{PublicKey, SecretKey};
use near_sdk::AccountId;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;

pub mod near_account;

use near_account::NearAccount;

#[derive(Debug)]
pub enum Account {
    Near(NearAccount),
    // TODO: Add Ethereum y Bitcoin
}

impl From<NearAccount> for Account {
    fn from(account: NearAccount) -> Self {
        Account::Near(account)
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct ConfigJSON {
    account_id: String,
    private_key: String,
    public_key: String,
}

const DEFAULT_ACCOUNTS_FILE_PATH: &str = "deployer.json";

pub fn get_user_account_info_from_file(
    config_file_path: Option<&str>,
) -> Result<NearAccount, Box<dyn std::error::Error>> {
    let path = config_file_path.unwrap_or(DEFAULT_ACCOUNTS_FILE_PATH);
    let mut file = File::open(path)?;
    // TODO: Change to support an array of configurations
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let config: ConfigJSON = serde_json::from_str(&contents)?;

    let account_id: AccountId = config.account_id.parse().unwrap();
    let private_key: SecretKey = config.private_key.parse().unwrap();
    let public_key: PublicKey = config.public_key.parse().unwrap();

    Ok(NearAccount {
        account_id,
        private_key,
        public_key,
    })
}
