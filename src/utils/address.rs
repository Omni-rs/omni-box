//! Utility functions to derive addresses for different networks
use bitcoin::hashes::{ripemd160, Hash};
use bitcoin::opcodes::all::{OP_CHECKSIG, OP_DUP, OP_EQUALVERIFY, OP_HASH160};
use bitcoin::script::Builder;
use bitcoin::{Address, CompressedPublicKey, Network};
use k256::elliptic_curve::sec1::FromEncodedPoint;
use k256::elliptic_curve::sec1::ToEncodedPoint;
use k256::sha2::{Digest, Sha256};
use k256::EncodedPoint;
use k256::{
    elliptic_curve::{bigint::ArrayEncoding, CurveArithmetic, PrimeField},
    AffinePoint, Scalar, Secp256k1, U256,
};
use near_sdk::AccountId;
use ripemd::Ripemd160;
use sha3::Sha3_256;
use tiny_keccak::{Hasher, Keccak};

// Types
pub type PublicKey = <Secp256k1 as CurveArithmetic>::AffinePoint;

trait ScalarExt: Sized {
    fn from_bytes(bytes: [u8; 32]) -> Option<Self>;
    fn from_non_biased(bytes: [u8; 32]) -> Self;
}

impl ScalarExt for Scalar {
    /// Returns nothing if the bytes are greater than the field size of Secp256k1.
    /// This will be very rare with random bytes as the field size is 2^256 - 2^32 - 2^9 - 2^8 - 2^7 - 2^6 - 2^4 - 1
    fn from_bytes(bytes: [u8; 32]) -> Option<Self> {
        let bytes = U256::from_be_slice(bytes.as_slice());
        Self::from_repr(bytes.to_be_byte_array()).into_option()
    }

    /// When the user can't directly select the value, this will always work
    /// Use cases are things that we know have been hashed
    fn from_non_biased(hash: [u8; 32]) -> Self {
        // This should never happen.
        // The space of inputs is 2^256, the space of the field is ~2^256 - 2^129.
        // This mean that you'd have to run 2^127 hashes to find a value that causes this to fail.
        Self::from_bytes(hash).expect("Derived epsilon value falls outside of the field")
    }
}

// Constant prefix that ensures epsilon derivation values are used specifically for
// near-mpc-recovery with key derivation protocol vX.Y.Z.
const EPSILON_DERIVATION_PREFIX: &str = "near-mpc-recovery v0.1.0 epsilon derivation:";

/// Derives an epsilon value from a given predecessor_id and path
pub fn derive_epsilon(predecessor_id: &AccountId, path: &str) -> Scalar {
    let derivation_path = format!("{EPSILON_DERIVATION_PREFIX}{},{}", predecessor_id, path);
    let mut hasher = Sha3_256::new();
    hasher.update(derivation_path);
    let hash: [u8; 32] = hasher.finalize().into();
    Scalar::from_non_biased(hash)
}

/// Derives a key from a given public key and epsilon value
pub fn derive_key(public_key: PublicKey, epsilon: Scalar) -> PublicKey {
    (<Secp256k1 as CurveArithmetic>::ProjectivePoint::GENERATOR * epsilon + public_key).to_affine()
}

const ROOT_PUBLIC_KEY: &str = "secp256k1:4NfTiv3UsGahebgTaHyD9vF8KYKMBnfd6kh94mK6xv8fGBiJB8TBtFMP5WWXz6B89Ac1fbpzPwAvoyQebemHFwx3";

/// Contains the derived address as string and the public key
/// that was used to derive the address
pub struct DerivedAddress {
    pub address: String,
    pub public_key: PublicKey,
}

/// Derives a Segwit address for a given path and predecessor_id
///
/// Example:
/// ```
/// let derived_address = get_derived_address_for_segwit("omnitester.testnet".parse().unwrap(), "bitcoin-1")
/// ```
pub fn get_derived_address_for_segwit(predecessor_id: &AccountId, path: &str) -> DerivedAddress {
    let epsilon = derive_epsilon(predecessor_id, path);
    let public_key = convert_string_to_public_key(ROOT_PUBLIC_KEY).unwrap();
    let derived_public_key = derive_key(public_key, epsilon);
    let address = public_key_to_btc_segwit_address(derived_public_key, "testnet");
    DerivedAddress {
        address,
        public_key: derived_public_key,
    }
}

/// Derives a Legacy address for a given path and predecessor_id
///
/// Example:
/// ```
/// let derived_address = get_derived_address_for_btc_legacy("omnitester.testnet".parse().unwrap(), "bitcoin-1")
/// ```
pub fn get_derived_address_for_btc_legacy(
    predecessor_id: &AccountId,
    path: &str,
) -> DerivedAddress {
    let epsilon = derive_epsilon(predecessor_id, path);
    let public_key = convert_string_to_public_key(ROOT_PUBLIC_KEY).unwrap();
    let derived_public_key = derive_key(public_key, epsilon);
    let btc_address = public_key_to_btc_address(derived_public_key, "testnet");

    DerivedAddress {
        address: btc_address,
        public_key: derived_public_key,
    }
}

/// Derives an EVM address for a given path and predecessor_id
///
/// Example:
/// ```
/// let derived_address = get_derived_address_for_evm("omnitester.testnet".parse().unwrap(), "ethereum-1")
/// ```
pub fn get_derived_address_for_evm(predecessor_id: &AccountId, path: &str) -> DerivedAddress {
    let epsilon = derive_epsilon(predecessor_id, path);
    let public_key = convert_string_to_public_key(ROOT_PUBLIC_KEY).unwrap();
    let derived_public_key = derive_key(public_key, epsilon);
    let evm_address = public_key_to_evm_address(derived_public_key);

    DerivedAddress {
        address: evm_address,
        public_key: derived_public_key,
    }
}

/// Obtains the public key bytes from a given derived address
pub fn get_public_key_as_bytes(derived_address: &DerivedAddress) -> Vec<u8> {
    let derived_public_key_bytes = derived_address.public_key.to_encoded_point(false); // Ensure this method exists
    let derived_public_key_bytes_array = derived_public_key_bytes.as_bytes();

    let bitcoin_pubkey = CompressedPublicKey::from_slice(derived_public_key_bytes_array)
        .expect("Invalid public key");

    bitcoin_pubkey.to_bytes().to_vec()
}

/// Obtains the public key hash from a given derived address
pub fn get_public_key_hash(derived_address: &DerivedAddress) -> Vec<u8> {
    // Create the public key from the derived address
    let derived_public_key_bytes = derived_address.public_key.to_encoded_point(false); // Ensure this method exists
    let derived_public_key_bytes_array = derived_public_key_bytes.as_bytes();

    let secp_pubkey = bitcoin::secp256k1::PublicKey::from_slice(derived_public_key_bytes_array)
        .expect("Invalid public key");

    let bitcoin_pubkey = bitcoin::PublicKey::new(secp_pubkey);

    let wpkh: bitcoin::WPubkeyHash = bitcoin_pubkey.wpubkey_hash().unwrap();

    wpkh.to_byte_array().to_vec()
}

/// Obtains the script_pub_key from a derived address
pub fn get_script_pub_key(derived_address: &DerivedAddress) -> Vec<u8> {
    let derived_public_key_bytes = derived_address.public_key.to_encoded_point(false); // Ensure this method exists
    let derived_public_key_bytes_array = derived_public_key_bytes.as_bytes();

    // Calculate publish key hash
    let sha256_hash = Sha256::digest(derived_public_key_bytes_array);
    let ripemd160_hash = ripemd160::Hash::hash(&sha256_hash);

    // The script_pubkey for the NEAR contract to be the spender
    let near_contract_script_pubkey = Builder::new()
        .push_opcode(OP_DUP)
        .push_opcode(OP_HASH160)
        .push_slice(ripemd160_hash.as_byte_array())
        .push_opcode(OP_EQUALVERIFY)
        .push_opcode(OP_CHECKSIG)
        .into_script();

    near_contract_script_pubkey.as_bytes().to_vec()
}

/// Obtains the script_sig from a derived address and a signature
pub fn build_script_sig_as_bytes(
    derived_address: DerivedAddress,
    signature: bitcoin::ecdsa::Signature,
) -> Vec<u8> {
    // Create the public key from the derived address
    let derived_public_key_bytes = derived_address.public_key.to_encoded_point(false); // Ensure this method exists
    let derived_public_key_bytes_array = derived_public_key_bytes.as_bytes();
    let secp_pubkey = bitcoin::secp256k1::PublicKey::from_slice(derived_public_key_bytes_array)
        .expect("Invalid public key");

    let bitcoin_pubkey = bitcoin::PublicKey::new_uncompressed(secp_pubkey);

    let script_sig_new = Builder::new()
        .push_slice(signature.serialize())
        .push_key(&bitcoin_pubkey)
        .into_script();

    script_sig_new.as_bytes().to_vec()
}

/// Converts a string-encoded public key to a public key (AffinePoint) non compressed
fn convert_string_to_public_key(encoded: &str) -> Result<PublicKey, String> {
    let base58_part = encoded.strip_prefix("secp256k1:").ok_or("Invalid prefix")?;

    let mut decoded_bytes = bs58::decode(base58_part)
        .into_vec()
        .map_err(|_| "Base58 decoding failed")?;

    if decoded_bytes.len() != 64 {
        return Err(format!(
            "Invalid public key length: expected 64, got {}",
            decoded_bytes.len()
        ));
    }

    decoded_bytes.insert(0, 0x04);

    let public_key = EncodedPoint::from_bytes(&decoded_bytes).unwrap();

    let public_key = AffinePoint::from_encoded_point(&public_key).unwrap();

    Ok(public_key)
}

#[allow(dead_code)]
fn public_key_to_hex(public_key: AffinePoint) -> String {
    let encoded_point = public_key.to_encoded_point(false);
    let encoded_point_bytes = encoded_point.as_bytes();

    hex::encode(encoded_point_bytes)
}

/// Converts a public key to a compressed public key
pub fn get_compressed_bitcoin_pubkey(derived_address: &DerivedAddress) -> Vec<u8> {
    let derived_public_key_bytes = derived_address.public_key.to_encoded_point(false);
    let derived_public_key_bytes_array = derived_public_key_bytes.as_bytes();

    let secp_pubkey = bitcoin::secp256k1::PublicKey::from_slice(derived_public_key_bytes_array)
        .expect("Invalid public key.");

    secp_pubkey.serialize().to_vec()
}

/// Converts a public key to an uncompressed public key
pub fn get_uncompressed_bitcoin_pubkey(derived_address: &DerivedAddress) -> Vec<u8> {
    let derived_public_key_bytes = derived_address.public_key.to_encoded_point(false); // no comprimida
    let derived_public_key_bytes_array = derived_public_key_bytes.as_bytes();

    let secp_pubkey = bitcoin::secp256k1::PublicKey::from_slice(derived_public_key_bytes_array)
        .expect("Invalid public key");

    secp_pubkey.serialize_uncompressed().to_vec()
}

/// Converts a public key to a Bitcoin address using P2PKH (Legacy)
fn public_key_to_btc_address(public_key: AffinePoint, network: &str) -> String {
    let encoded_point = public_key.to_encoded_point(false);
    let public_key_bytes = encoded_point.as_bytes();

    let sha256_hash = Sha256::digest(public_key_bytes);

    let ripemd160_hash = Ripemd160::digest(sha256_hash);

    let network_byte = if network == "bitcoin" { 0x00 } else { 0x6f };
    let mut address_bytes = vec![network_byte];
    address_bytes.extend_from_slice(&ripemd160_hash);

    base58check_encode(&address_bytes)
}

/// Converts a public key to an Ethereum address
pub fn public_key_to_evm_address(public_key: AffinePoint) -> String {
    let encoded_point = public_key.to_encoded_point(false);
    let public_key_bytes = encoded_point.as_bytes();

    // Exclude the first byte (0x04 prefix) and take only the X part
    let x_only = &public_key_bytes[1..];

    // Calculate the hash Keccak-256
    let mut keccak = Keccak::v256();
    let mut hash = [0u8; 32];
    keccak.update(x_only);
    keccak.finalize(&mut hash);

    // Take the last 20 bytes
    let eth_address_bytes = &hash[12..];

    format!("0x{}", hex::encode(eth_address_bytes))
}

/// Converts a public key to a public key hash
pub fn public_key_to_hash(public_key: AffinePoint) -> Vec<u8> {
    let encoded_point = public_key.to_encoded_point(false);
    let public_key_bytes = encoded_point.as_bytes();

    let compressed_pubkey =
        CompressedPublicKey::from_slice(public_key_bytes).expect("Invalid pubkey");

    let pubkey_hash = compressed_pubkey.wpubkey_hash();

    pubkey_hash.to_byte_array().to_vec()
}

/// Converts a public key to a Bitcoin address using P2WPKH (SegWit)
pub fn public_key_to_btc_segwit_address(public_key: AffinePoint, network: &str) -> String {
    let encoded_point = public_key.to_encoded_point(false);
    let public_key_bytes = encoded_point.as_bytes();

    let compressed_pubkey =
        CompressedPublicKey::from_slice(public_key_bytes).expect("Invalid pubkey");

    let network = if network == "testnet" {
        Network::Regtest
    } else {
        Network::Bitcoin
    };

    let segwit_address = Address::p2wpkh(&compressed_pubkey, network);

    segwit_address.to_string()
}

fn base58check_encode(data: &[u8]) -> String {
    // Perform a double SHA-256 hash on the data
    let hash1 = Sha256::digest(data);
    let hash2 = Sha256::digest(hash1);

    // Take the first 4 bytes of the second hash as the checksum
    let checksum = &hash2[..4];

    // Append the checksum to the original data
    let mut data_with_checksum = Vec::from(data);
    data_with_checksum.extend_from_slice(checksum);

    // Encode the data with checksum using Base58
    bs58::encode(data_with_checksum).into_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_epsilon() {
        let predecessor_id = "omnitester.testnet".parse().unwrap();
        let path = "bitcoin-1";

        let epsilon = derive_epsilon(&predecessor_id, path);

        let public_key = convert_string_to_public_key("secp256k1:4NfTiv3UsGahebgTaHyD9vF8KYKMBnfd6kh94mK6xv8fGBiJB8TBtFMP5WWXz6B89Ac1fbpzPwAvoyQebemHFwx3").unwrap();

        let derived_public_key = derive_key(public_key, epsilon);

        let derived_public_key_hex = public_key_to_hex(derived_public_key);

        let btc_address = public_key_to_btc_address(derived_public_key, "testnet");

        assert_eq!(btc_address, "n19iEMJE2L2YBfJFsXC8Gzs7Q2Z7TwdCqv");
        assert_eq!(derived_public_key_hex, "0471f75dc56b971fbe52dd3e80d2f8532eb8905157556df39cb7338a67c80412640c869f717217ba5b916db6d7dc7d6a84220f8251e626adad62cac9c7d6f8e032");
    }

    #[test]
    fn test_evm_address() {
        let predecessor_id = "omnitester.testnet".parse().unwrap();
        let path = "ethereum-1";

        let epsilon = derive_epsilon(&predecessor_id, path);

        let public_key = convert_string_to_public_key("secp256k1:4NfTiv3UsGahebgTaHyD9vF8KYKMBnfd6kh94mK6xv8fGBiJB8TBtFMP5WWXz6B89Ac1fbpzPwAvoyQebemHFwx3").unwrap();

        let derived_public_key = derive_key(public_key, epsilon);

        let derived_public_key_hex = public_key_to_hex(derived_public_key);

        let evm_address = public_key_to_evm_address(derived_public_key);

        assert_eq!(evm_address, "0xd8d25820c9b9e2aa9cce55504355e500efcce715");
        assert_eq!(derived_public_key_hex, "04e612e7650febebc50b448bf790f6bdd70a8a6ce3b111a1d7e72c87afe84be776e36226e3f89de1ba3cbb62c0f3fc05bffae672c9c59d5fa8a4737b6547c64eb7");
    }
}
