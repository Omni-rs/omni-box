use near_crypto::{PublicKey, SecretKey};
use near_sdk::AccountId;

#[derive(Debug, Clone)]
pub struct NearAccount {
    pub account_id: AccountId,
    pub private_key: SecretKey,
    pub public_key: PublicKey,
}
