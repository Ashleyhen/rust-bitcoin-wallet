use std::fmt::Display;

use serde::{Deserialize, Serialize};


#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WalletTestVectors {
    pub version: u64,
    #[serde(
        rename = "scriptPubKey",
        skip_serializing_if = "Vec::is_empty",
        default
    )]
    pub script_pub_key: Vec<ScriptPubKey>,
#[serde(
        rename = "keyPathSpending",
        skip_serializing_if = "Vec::is_empty",
        default
    )]
    pub key_spending_path:Vec<KeyPathSpending>
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScriptPubKey {
    pub given: Given,
    pub intermediary: Intermediary,
    pub expected: Expected,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScriptTree {
    pub id: u64,
    pub script: String,
    #[serde(rename = "leafVersion")]
    pub leaf_version: u64,
}

// TODO
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Intermediary {
    #[serde(rename = "hashAmounts")]
    pub hash_amounts: Option<String>,
    #[serde(rename = "hashOutputs")]
    pub hash_outputs: Option<String>,
    #[serde(rename = "hashPrevouts")]
    pub hash_prevouts: Option<String>,
    #[serde(rename = "hashScriptPubkeys")]
    pub hash_script_pubkeys: Option<String>,
    #[serde(rename = "hashSequences")]
    pub hash_sequences: Option<String>,
    #[serde(rename = "internalPubkey")]
    pub internal_pubkey: Option<String>,
    #[serde(rename = "leafHashes", skip_serializing_if = "Vec::is_empty", default)]
    pub leaf_hashes: Vec<String>,
    #[serde(rename = "merkleRoot")]
    pub merkle_root: Option<String>,
    #[serde(
        rename = "precomputedUsed",
        skip_serializing_if = "Vec::is_empty",
        default
    )]
    pub precomputed_used: Vec<String>,
    #[serde(rename = "sigHash")]
    pub sig_hash: Option<String>,
    #[serde(rename = "sigMsg")]
    pub sig_msg: Option<String>,
    #[serde(rename = "tweakedPrivkey")]
    pub tweaked_privkey: Option<String>,
    #[serde(rename = "tweakedPubkey")]
    pub tweaked_pubkey: Option<String>,
    pub tweak: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Expected {
    #[serde(rename = "scriptPubKey")]
    pub script_pub_key: Option<String>,
    #[serde(rename = "bip350Address")]
    pub bip350address: Option<String>,
    #[serde(
        rename = "scriptPathControlBlocks",
        skip_serializing_if = "Vec::is_empty",
        default
    )]
    pub script_path_control_blocks: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub witness: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Root {
    #[serde(rename = "keyPathSpending")]
    pub key_path_spending: Vec<KeyPathSpending>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyPathSpending {
    pub given: Given,
    pub intermediary: Intermediary,
    #[serde(rename = "inputSpending")]
    pub input_spending: Vec<InputSpending>,
    pub auxiliary: Auxiliary,
}

// TODO
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Given {
    #[serde(rename = "hashType")]
    pub hash_type: Option<u8>,
    #[serde(rename = "internalPrivkey")]
    pub internal_privkey: Option<String>,
    #[serde(rename = "internalPubkey")]
    pub internal_pubkey: Option<String>,
    #[serde(rename = "merkleRoot")]
    pub merkle_root: Option<String>,
    #[serde(rename = "rawUnsignedTx")]
    pub raw_unsigned_tx: Option<String>,
    #[serde(rename = "scriptTree", skip_serializing_if = "Vec::is_empty", default)]
    pub script_tree: Vec<ScriptTree>,
    #[serde(rename = "txinIndex")]
    pub txin_index: Option<u64>,
    #[serde(rename = "utxosSpent", skip_serializing_if = "Vec::is_empty", default)]
    pub utxos_spent: Vec<UtxosSpent>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UtxosSpent { 
    #[serde(rename = "scriptPubKey")]
    pub script_pub_key: String,
    #[serde(rename = "amountSats")]
    pub amount_sats: u64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InputSpending {
    pub given: Given,
    pub intermediary: Intermediary,
    pub expected: Expected,
}

// TODO
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Auxiliary {
    #[serde(rename = "fullySignedTx")]
    pub fully_signed_tx: String,
}
