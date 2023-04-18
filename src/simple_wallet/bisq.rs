use bitcoin::{blockdata::{script::Builder, opcodes::all::OP_CHECKSIGADD},  Script, util::taproot::TaprootBuilder, psbt::TapTree, secp256k1::PublicKey, XOnlyPublicKey, Address };

use crate::bitcoin_wallet::constants::{secp, NETWORK};

pub fn unlock_bond(host:&XOnlyPublicKey,client:&XOnlyPublicKey)->Script{
    Builder::new()
        .push_x_only_key(&host)
        .push_x_only_key(&client)
        .push_opcode(OP_CHECKSIGADD).into_script()
}
pub fn unlock_support(support_key:&XOnlyPublicKey)->Script{
    Builder::new()
        .push_x_only_key(support_key)
        .push_opcode(OP_CHECKSIGADD).into_script()
}
pub fn create_address(host:&XOnlyPublicKey,client:&XOnlyPublicKey,support_key:&XOnlyPublicKey){
    // vec![]
    let combined_scripts=vec![
        (1,unlock_bond(host,client)),
        (1,unlock_support(support_key))
        ];
    let tap_tree=TaprootBuilder::with_huffman_tree(combined_scripts).unwrap();

    let tap_root_spend_info =TapTree::try_from(tap_tree).unwrap().into_builder()
    .finalize(&secp(), *support_key).unwrap();

    Address::p2tr(&secp(), *support_key, tap_root_spend_info.merkle_root(), NETWORK);
}

