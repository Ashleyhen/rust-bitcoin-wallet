use std::{env, str::FromStr};

use bitcoin::{
    blockdata::{opcodes::all, script::Builder},
    hashes::hex::FromHex,
    psbt::{Input, Output},
    secp256k1::{Secp256k1, SecretKey},
    util::{
        bip32::{ExtendedPrivKey, ExtendedPubKey},
        taproot::ControlBlock,
    },
    Address, KeyPair, Script, Transaction, TxIn, TxOut, XOnlyPublicKey,
};
use bitcoin_hashes::Hash;
use bitcoin_wallet::{
    address_formats::{derive_derivation_path, map_seeds_to_scripts, map_tr_address},
    constants::NETWORK,
    input_data::{electrum_rpc::ElectrumRpc, tapscript_ex_input::TapscriptExInput, RpcCall},
    script_services::psbt_factory::create_partially_signed_tx,
    spending_path::{p2tr_key_path::P2TR_K, tap_script_spending_ex::TapScriptSendEx},
};

use either::Either;
use miniscript::{psbt::PsbtExt, ToPublicKey};
use wallet_test::{tapscript_example_with_tap::Test, wallet_test_vector_traits::WalletTestVectors};

use crate::bitcoin_wallet::{
    address_formats::{p2tr_addr_fmt::P2TR, AddressSchema},
    input_data::{reuse_rpc_call::ReUseCall, tapscript_ex_input::get_signed_tx},
    wallet_methods::{BroadcastOp, ClientWithSchema},
};

pub mod wallet_test;

pub mod bitcoin_wallet;

fn main() {
    env::set_var("RUST_BACKTRACE", "full");
    // key_tx();
    // script_tx();

     Test();
}

pub fn key_tx() {
    let secp = Secp256k1::new();
    let seed = "1d454c6ab705f999d97e6465300a79a9595fb5ae1186ae20e33e12bea606c094";
    let tr = vec![
        "tb1puma0fas8dgukcvhm8ewsganj08edgnm6ejyde3ev5lvxv4h7wqvqpjslxz".to_string(),
        "tb1phtgnyv6qj4n6kqkmm2uzg630vz2tmgv4kchdp44j7my6qre4qdys6hchvx".to_string(),
        "tb1p95xjusgkgh2zqhyr5q9hzwv607yc5dncnsastm9xygmmuu4xrcqs53468m".to_string(),
        "tb1pz6egnzpq0h92zjkv23vdt4gwy8thd4t0t66megj20cr32m64ds4qv2kcal".to_string(),
        "tb1p69eefuuvaalsdljjyqntnrrtc4yzpc038ujm3ppze8g6ljepskks2zzffj".to_string(),
    ];
    let derivation_fn = derive_derivation_path(
        ExtendedPrivKey::new_master(NETWORK, &SecretKey::from_str(seed).unwrap().secret_bytes())
            .unwrap(),
        341,
    );
    let private_ext_keys = (0..5)
        .map(|index| derivation_fn(0, index))
        .collect::<Vec<ExtendedPrivKey>>();
    let key_pair = private_ext_keys
        .iter()
        .map(|prv| KeyPair::from_secret_key(&secp, prv.private_key))
        .collect::<Vec<KeyPair>>();
    let address_generate = map_tr_address(None);
    let addresses = private_ext_keys
        .iter()
        .map(|f| address_generate(&secp, ExtendedPubKey::from_priv(&secp, f)))
        .collect::<Vec<Address>>();
    let my_add = addresses[3].script_pubkey();
    let electrum = ElectrumRpc::new(&my_add);
    let tr = P2TR_K::new(&secp);
    let send_addr = "tb1p5kaqsuted66fldx256lh3en4h9z4uttxuagkwepqlqup6hw639gskndd0z".to_string();
    let output_func = tr.output_factory(
        Address::from_str(&send_addr).unwrap().script_pubkey(),
        addresses[3].script_pubkey(),
    );
    let unlock_func = tr.input_factory(&key_pair[3]);
    let psbt =
        create_partially_signed_tx(output_func, P2TR_K::create_tx(10000), unlock_func)(&electrum);
    let finalize = psbt.finalize(&secp).unwrap().extract_tx();
    // dbg!(Address::from_script(&finalize.output[1].script_pubkey, NETWORK).unwrap().to_string());
    // dbg!(Address::from_script(&finalize.output[0].script_pubkey, NETWORK).unwrap().to_string());
    dbg!(finalize.clone());
    let tx = finalize.clone();
}

pub fn script_tx() {
    let seed = "1d454c6ab705f999d97e6465300a79a9595fb5ae1186ae20e33e12bea606c094";
    let alice_secret = 0;
    let bob_secret = 1;
    let internal_secret = 2;
    let secp = Secp256k1::new();
    let seeds = vec![
        "2bd806c97f0e00af1a1fc3328fa763a9269723c8db8fac4f93af71db186d6e90", //alice
        "81b637d8fcd2c6da6359e6963113a1170de795e4b725b84d1e0b4cfd9ec58ce9", //bob
        "1229101a0fcf2104e8808dab35661134aa5903867d44deb73ce1c7e4eb925be8", //internal
    ];

    let keys = seeds
        .iter()
        .map(|scrt| KeyPair::from_secret_key(&secp, SecretKey::from_str(&scrt).unwrap()))
        .collect::<Vec<KeyPair>>();

    let tap_script = TapScriptSendEx::new(&secp);
    let tap_key = P2TR_K::new(&secp);

    let addr_generator =
        map_seeds_to_scripts(Some(seed.to_string()), &secp, 341, map_tr_address(None));
    let addr_list = (0..5)
        .map(|i| addr_generator(0, i))
        .collect::<Vec<Address>>();
    let my_add = Address::from_str(
        &"tb1p5kaqsuted66fldx256lh3en4h9z4uttxuagkwepqlqup6hw639gskndd0z".to_string(),
    )
    .unwrap()
    .script_pubkey();

    let output_func = tap_key.single_output(addr_list[3].script_pubkey());
    let electrum = ElectrumRpc::new(&my_add);
    let lock_func = TapScriptSendEx::create_tx();
    let unlock_func =
        tap_script.input_factory(&keys[bob_secret], keys[internal_secret].public_key());
    let psbt =
        create_partially_signed_tx(vec![vec![output_func]], lock_func, unlock_func)(&electrum);
    let tx = TapScriptSendEx::finialize_script(psbt, &keys[bob_secret].public_key());
    let tx_id=electrum.transaction_broadcast(tx);
    dbg!(tx_id);
    //
}
