use crate::clients::get_bitcoin_instance;

use crate::utils::address::DerivedAddress;
use bitcoin::bip32::DerivationPath;
use bitcoin::secp256k1::{PublicKey, Secp256k1, SecretKey};
use bitcoin::{bip32::Xpriv, Address, Network, ScriptBuf};
use bitcoin::{CompressedPublicKey, PublicKey as BitcoinPublicKey, WPubkeyHash};
use bitcoind::AddressType;
use serde_json::{json, Value};
use std::str::FromStr as _;

#[derive(Debug)]
pub struct UserInfo {
    pub address: Address,
    pub script_pubkey: ScriptBuf,
    pub private_key: SecretKey,
    pub public_key: PublicKey,
    pub compressed_public_key: CompressedPublicKey,
    pub bitcoin_public_key: BitcoinPublicKey,
    pub wpkh: WPubkeyHash,
}

#[derive(Debug)]
pub struct BTCTestContext {
    pub bitcoind_instance: bitcoind::BitcoinD,
    pub master_key_p2pkh: Xpriv,
    pub master_key_p2wpkh: Xpriv,
    pub alice_legacy: UserInfo,
    pub alice_segwit: UserInfo,
    pub bob_legacy: UserInfo,
    pub bob_segwit: UserInfo,
}

impl Default for BTCTestContext {
    fn default() -> Self {
        let bitcoind: bitcoind::BitcoinD = get_bitcoin_instance().unwrap();
        Self::new(bitcoind).unwrap()
    }
}

impl BTCTestContext {
    pub fn new(bitcoind_instance: bitcoind::BitcoinD) -> Result<Self, Box<dyn std::error::Error>> {
        let client = &bitcoind_instance.client;
        let master_key_p2pkh = Self::get_master_key_of_regtest_node_p2pkh(client)?;
        let master_key_p2wpkh = Self::get_master_key_of_regtest_node_p2wpkh(client)?;

        let alice_legacy =
            Self::setup_account(client, master_key_p2pkh, AddressType::Legacy).unwrap();
        let alice_segwit =
            Self::setup_account(client, master_key_p2wpkh, AddressType::Bech32).unwrap();
        let bob_legacy =
            Self::setup_account(client, master_key_p2pkh, AddressType::Legacy).unwrap();
        let bob_segwit =
            Self::setup_account(client, master_key_p2wpkh, AddressType::Bech32).unwrap();

        Ok(Self {
            bitcoind_instance,
            master_key_p2pkh,
            master_key_p2wpkh,
            alice_legacy,
            alice_segwit,
            bob_legacy,
            bob_segwit,
        })
    }

    pub const fn client(&self) -> &bitcoind::Client {
        &self.bitcoind_instance.client
    }

    fn setup_account(
        client: &bitcoind::Client,
        master_key_p2pkh_or_p2wpkh: Xpriv,
        address_type: AddressType,
    ) -> Result<UserInfo, Box<dyn std::error::Error>> {
        let address = client
            .get_new_address_with_type(address_type.clone())
            .unwrap()
            .address()
            .unwrap();

        let address = address.require_network(Network::Regtest).unwrap();

        // Get address info for Account
        let address_info: Value = client.call("getaddressinfo", &[address.to_string().into()])?;

        // Extract the pubkey from the address info
        let pubkey_hex = address_info["pubkey"]
            .as_str()
            .expect("pubkey should be a string");

        let compressed_pub_key =
            CompressedPublicKey::from_str(pubkey_hex).expect("Failed to parse pubkey");

        // Extract the scriptPubKey from the address info
        let script_pubkey_hex = address_info["scriptPubKey"]
            .as_str()
            .expect("scriptPubKey should be a string");

        let script_pubkey =
            ScriptBuf::from_hex(script_pubkey_hex).expect("Failed to parse scriptPubKey");

        // Initialize secp256k1 context
        let secp = Secp256k1::new();

        // Derive child private key using path m/44h/1h/0h
        let hd_key_path = address_info["hdkeypath"].as_str().unwrap();
        let path = DerivationPath::from_str(hd_key_path).unwrap();

        // let child = if address_type == AddressType::Bech32 {
        //     self.master_key_p2wpkh.derive_priv(&secp, &path).unwrap()
        // } else {
        //     self.master_key_p2pkh.derive_priv(&secp, &path).unwrap()
        // };
        let child = master_key_p2pkh_or_p2wpkh
            .derive_priv(&secp, &path)
            .unwrap();

        let private_key = child.private_key;
        let public_key = PublicKey::from_secret_key(&secp, &private_key);
        let bitcoin_public_key = BitcoinPublicKey::new(public_key);

        let derived_address = if address_type == AddressType::Bech32 {
            Address::p2wpkh(&compressed_pub_key, Network::Regtest)
        } else {
            Address::p2pkh(compressed_pub_key, Network::Regtest)
        };

        assert_eq!(
            bitcoin_public_key.to_string(),
            pubkey_hex,
            "Derived public key does not match the one provided by the node"
        );
        // Verify that the address is the same as the one generated by the client
        assert_eq!(address, derived_address);

        let wpkh: WPubkeyHash = bitcoin_public_key
            .wpubkey_hash()
            .expect("Failed to compute WPubkeyHash: ensure the key is compressed");

        Ok(UserInfo {
            address,
            script_pubkey,
            private_key,
            public_key,
            bitcoin_public_key,
            wpkh,
            compressed_public_key: compressed_pub_key,
        })
    }

    pub fn create_account(
        &self,
        address_type: AddressType,
    ) -> Result<UserInfo, Box<dyn std::error::Error>> {
        let client = self.client();
        let master_key = if address_type == AddressType::Bech32 {
            self.master_key_p2wpkh
        } else {
            self.master_key_p2pkh
        };
        Self::setup_account(client, master_key, address_type)
    }

    pub fn generate_to_derived_address(
        &self,
        derived_address: &DerivedAddress,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let near_contract_address =
            bitcoin::Address::from_str(&derived_address.address.to_string())?;
        let near_contract_address = near_contract_address
            .require_network(Network::Regtest)
            .unwrap();

        self.client()
            .generate_to_address(101, &near_contract_address)?;

        Ok(())
    }

    fn get_master_key_of_regtest_node_p2pkh(
        client: &bitcoind::Client,
    ) -> Result<Xpriv, Box<dyn std::error::Error>> {
        let descriptors: Value = client.call("listdescriptors", &[true.into()])?;

        let p2pkh_descriptor = descriptors["descriptors"]
            .as_array()
            .unwrap()
            .iter()
            .find(|descriptor| descriptor["desc"].as_str().unwrap().contains("pkh"))
            .expect("No P2PKH descriptor found");

        let desc = p2pkh_descriptor["desc"].as_str().unwrap();
        let parts: Vec<&str> = desc.split('/').collect();
        let master_key_str = parts[0].replace("pkh(", "").replace(")", "");

        let master_key = Xpriv::from_str(&master_key_str).unwrap();

        Ok(master_key)
    }

    fn get_master_key_of_regtest_node_p2wpkh(
        client: &bitcoind::Client,
    ) -> Result<Xpriv, Box<dyn std::error::Error>> {
        let descriptors: Value = client.call("listdescriptors", &[true.into()])?;

        let p2wpkh_descriptor = descriptors["descriptors"]
            .as_array()
            .unwrap()
            .iter()
            .find(|descriptor| {
                let desc = descriptor["desc"].as_str().unwrap();
                desc.contains("wpkh") && !desc.starts_with("tr(") // Exclude descriptors for taproot
            })
            .expect("No P2WPKH or nested P2WPKH descriptor found");

        let desc = p2wpkh_descriptor["desc"].as_str().unwrap();

        // Extract the xpriv part from the descriptor
        let xpriv_part = desc
            .split("wpkh(")
            .nth(1)
            .unwrap()
            .split(')')
            .next()
            .unwrap();
        let parts: Vec<&str> = xpriv_part.split('/').collect();
        let master_key_str = parts[0];

        // Ensure the key starts with "tprv" for testnet/regtest
        let master_key_str = if !master_key_str.starts_with("tprv") {
            format!("tprv{}", master_key_str)
        } else {
            master_key_str.to_string()
        };

        let master_key = Xpriv::from_str(&master_key_str)?;

        Ok(master_key)
    }

    pub fn scan_utxo_for_address(
        &self,
        address: &DerivedAddress,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let near_contract_address = bitcoin::Address::from_str(&address.address.to_string())?;
        let near_contract_address = near_contract_address
            .require_network(Network::Regtest)
            .unwrap();

        let scan_txout_set_result: serde_json::Value = self
            .client()
            .call(
                "scantxoutset",
                &[
                    json!("start"),
                    json!([{ "desc": format!("addr({})", near_contract_address) }]),
                ],
            )
            .unwrap();

        Ok(scan_txout_set_result)
    }

    pub fn scan_utxo_for_address_with_count(
        &self,
        address: &DerivedAddress,
        count: usize,
    ) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
        let near_contract_address = bitcoin::Address::from_str(&address.address.to_string())?;
        let near_contract_address = near_contract_address
            .require_network(Network::Regtest)
            .unwrap();

        let scan_txout_set_result: serde_json::Value = self
            .client()
            .call(
                "scantxoutset",
                &[
                    json!("start"),
                    json!([{ "desc": format!("addr({})", near_contract_address) }]),
                ],
            )
            .unwrap();

        // Extraer los outputs no gastados
        let unspents = scan_txout_set_result
            .as_object()
            .unwrap()
            .get("unspents")
            .unwrap()
            .as_array()
            .unwrap();

        // Obtener la cantidad solicitada de elementos
        let selected_unspents: Vec<serde_json::Value> =
            unspents.iter().take(count).cloned().collect();

        Ok(selected_unspents)
    }

    pub fn assert_utxos_for_address(&self, address: Address, number_of_utxos: usize) {
        let unspent_utxos: Vec<serde_json::Value> = self.get_utxo_for_address(&address).unwrap();

        assert!(
            unspent_utxos.len() == number_of_utxos,
            "Expected {} UTXOs for address {}, but found {}",
            number_of_utxos,
            address,
            unspent_utxos.len()
        );
    }

    pub fn get_utxo_for_address(
        &self,
        address: &Address,
    ) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
        let min_conf = 1;
        let max_conf = 9999999;
        let include_unsafe = true;
        let query_options = json!({});

        let unspent_utxos: Vec<serde_json::Value> = self.client().call(
            "listunspent",
            &[
                json!(min_conf),
                json!(max_conf),
                json!(vec![address.to_string()]),
                json!(include_unsafe),
                query_options,
            ],
        )?;

        // Verify UTXO belongs to the address and has the correct amount
        for utxo in unspent_utxos.iter() {
            assert_eq!(
                utxo["address"].as_str().unwrap(),
                address.to_string(),
                "UTXO doesn't belong to the address"
            );
        }

        Ok(unspent_utxos)
    }

    pub const fn get_alice_legacy(&self) -> &UserInfo {
        &self.alice_legacy
    }

    pub const fn get_alice_segwit(&self) -> &UserInfo {
        &self.alice_segwit
    }

    pub const fn get_bob_legacy(&self) -> &UserInfo {
        &self.bob_legacy
    }

    pub const fn get_bob_segwit(&self) -> &UserInfo {
        &self.bob_segwit
    }
}
