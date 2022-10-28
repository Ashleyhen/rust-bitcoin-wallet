use std::str::FromStr;

use crate::bitcoin_wallet::{
    address_formats::{derive_derivation_path, generate_key_pair},
    constants::{NETWORK, SEED},
    input_data::regtest_rpc::RegtestRpc,
    script_services::psbt_factory::{
        create_partially_signed_tx, default_output, get_output, merge_psbt,
    },
    spending_path::{
        musig::{self, MuSig_Script},
        p2tr_key_path::P2tr,
        tap_script_spending_ex::TapScriptSendEx,
    },
};
use bitcoin::{
    secp256k1::Secp256k1,
    util::bip32::{ExtendedPrivKey, ExtendedPubKey},
    Address,
};

pub fn musig_demo() {
    let secp = Secp256k1::new();
    let get_xonly_fn = |i, j| {
        derive_derivation_path(generate_key_pair(Some(SEED.to_string())), 32)(j, i)
            .to_keypair(&secp)
    };
    let x_x_only_1 = get_xonly_fn(0, 0).public_key().x_only_public_key().0;
    let i_x_only_1 = get_xonly_fn(0, 1).public_key().x_only_public_key().0;
    let internal = get_xonly_fn(0, 2).public_key().x_only_public_key().0;
    let key = get_xonly_fn(0, 0);
    let ext_pub = ExtendedPubKey::from_priv(
        &secp,
        &ExtendedPrivKey::new_master(NETWORK, &key.secret_bytes()).unwrap(),
    );
    let len = ext_pub.public_key.serialize_uncompressed().len();
    let x_x_only_2 = get_xonly_fn(1, 0).public_key().x_only_public_key().0;
    let i_x_only_2 = get_xonly_fn(1, 1).public_key().x_only_public_key().0;

    let mut party_1 = MuSig_Script::broadcast(&x_x_only_1, &i_x_only_1);
    let mut party_2 = MuSig_Script::broadcast(&x_x_only_2, &i_x_only_2);
    party_1.append(&mut party_2);

    MuSig_Script::compute_challenge(&party_1);
    // tap_sign.input_factory(bob_keypair: &'a KeyPair, internal_key: XOnlyPublicKey);
}
