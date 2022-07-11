use bitcoin::{
    blockdata::{opcodes, script::Builder},
    util::{bip32::ExtendedPubKey, taproot::TaprootBuilder},
    Address, Script, TxOut, XOnlyPublicKey,
};
use electrum_client::ListUnspentRes;
use std::{borrow::BorrowMut, ops::Add, str::FromStr, sync::Arc};

use super::{wallet_methods::NETWORK, wallet_traits::AddressSchema};

pub fn pub_key_lock<'a, S>(
    schema: &'a S,
    amount: u64,
    total: u64,
    change_addr: ExtendedPubKey,
    to_addr: String,
) -> Vec<TxOut>
where
    S: AddressSchema,
{
    let tip: u64 = 300;

    let send_tx = TxOut {
        value: amount,
        script_pubkey: Address::from_str(&to_addr).unwrap().script_pubkey(),
    };

    if (total <= (amount + tip)) {
        return vec![send_tx];
    }

    let change_tx = TxOut {
        value: total - (amount + tip),
        script_pubkey: schema.map_ext_keys(&change_addr).script_pubkey(),
    };

    return vec![send_tx, change_tx];
}

fn dynamic_builder(mut iter: impl Iterator<Item = XOnlyPublicKey>) -> Builder {
    return match iter.next() {
        Some(data) => dynamic_builder(iter)
            .push_x_only_key(&data)
            .push_opcode(opcodes::all::OP_CHECKSIGADD),
        None => Builder::new(),
    };
}

pub fn multi_sig_lock<'a, S>(
    schema: &'a S,
    amount: u64,
    total: u64,
    change_addr: ExtendedPubKey,
    to_addr: Vec<String>,
) -> Vec<TxOut>
where
    S: AddressSchema,
{
    let tip: u64 = 300;

    let wallet_keys = schema.to_wallet().create_wallet(
        schema.wallet_purpose(),
        schema.to_wallet().recieve,
        schema.to_wallet().change,
    );

    schema.map_ext_keys(&wallet_keys.0).script_pubkey();

    let script = dynamic_builder(to_addr.iter().map(|addr| {
        XOnlyPublicKey::from_slice(&Address::from_str(&addr).unwrap().script_pubkey()[2..]).unwrap()
    }))
    .push_int(2)
    .push_opcode(opcodes::all::OP_EQUAL)
    .into_script();

    let trap = TaprootBuilder::new().add_leaf(0, script).unwrap();
    let internal =
        XOnlyPublicKey::from_slice(&Address::from_str(&to_addr[0]).unwrap().script_pubkey()[2..])
            .unwrap();

    let script_pub_k = Script::new_v1_p2tr(
        &schema.to_wallet().secp,
        internal,
        trap.finalize(&schema.to_wallet().secp, internal)
            .unwrap()
            .merkle_root(),
    ); // TaprootMerkleBranch

    let send_tx = TxOut {
        value: amount,
        script_pubkey: script_pub_k,
    };

    if (total <= (amount + tip)) {
        return vec![send_tx];
    }

    let change_tx = TxOut {
        value: total - (amount + tip),
        script_pubkey: schema.map_ext_keys(&change_addr).script_pubkey(),
    };

    return vec![send_tx, change_tx];
}
