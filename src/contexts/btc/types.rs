use omni_transaction::bitcoin::types::{Amount, Hash, Txid};
use serde::de::{self};
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ListUnspentResult {
    pub txid: String,
    pub vout: u32,
    pub address: String,
    pub label: String,
    pub script_pubkey: String,
    pub amount: f64,
    pub confirmations: u32,
    pub redeem_script: Option<String>,
    pub witness_script: Option<String>,
    pub spendable: bool,
    pub solvable: bool,
    pub reused: Option<bool>,
    pub desc: Option<String>,
    pub safe: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScanTxOutSetResult {
    pub success: bool,
    pub txouts: u32,
    pub height: u32,
    pub bestblock: String,
    pub unspents: Vec<UnspentOutput>,
    pub total_amount: f64,
}

#[derive(Debug, Serialize)]
pub struct UnspentOutput {
    pub txid: Txid,
    pub vout: u32,
    pub script_pubkey: String,
    pub desc: String,
    pub amount: Amount,
    pub height: u32,
}

impl<'de> Deserialize<'de> for UnspentOutput {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct UnspentOutputHelper {
            txid: String,
            vout: u32,
            script_pubkey: String,
            desc: String,
            amount: f64,
            height: u32,
        }

        let helper = UnspentOutputHelper::deserialize(deserializer)?;
        let tx_hash = Hash::from_hex(&helper.txid).map_err(de::Error::custom)?;
        let txid = Txid(tx_hash);
        let amount = Amount::from_sat((helper.amount * 100_000_000.0) as u64);

        Ok(UnspentOutput {
            txid,
            vout: helper.vout,
            script_pubkey: helper.script_pubkey,
            desc: helper.desc,
            amount,
            height: helper.height,
        })
    }
}
