use std::str::FromStr;

use bitcoin::{secp256k1::Secp256k1, Address};
use miniscript::psbt::PsbtExt;

use crate::bitcoin_wallet::{address_formats::{derive_derivation_path, generate_key_pair}, constants::SEED, spending_path::{musig::MuSig, tap_script_spending_ex::TapScriptSendEx, p2tr_key_path::P2tr}, script_services::psbt_factory::{default_output, get_output, create_partially_signed_tx, merge_psbt}, input_data::regtest_rpc::RegtestRpc};


pub fn musig_demo(){
let secp = Secp256k1::new();
    let get_xonly_fn=|i|derive_derivation_path(generate_key_pair(Some(SEED.to_string())),32)
    (0,i).to_keypair(&secp);
    let internal =get_xonly_fn(0).public_key();
    let x_only=get_xonly_fn(1).public_key();//bcrt1pcfc3whp89g2va7zwndpzjgdlzr4x03754nln0n0lvnj6rupm6pjq8xkj0r
    let x_only_2=get_xonly_fn(2).public_key();//bcrt1p9pfqwxwe5v25mw5452nswzvwp6qnpk3q8hjduvc2p6sllr6plfjsjls0z5
    let musig=MuSig::new(&secp);
    let output_vec_vec_func= ||vec![musig.merge(x_only, x_only_2)];

    TapScriptSendEx::get_script_addresses(get_output(output_vec_vec_func(),&mut default_output()))
        .iter()
        .for_each(|f| println!("target address {}", f.to_string()));

        let p2tr=P2tr::new(&secp);

        let target_addr="bcrt1pnd0alw2mgssggn09yva0htpq0syrkeug92ryjd2xv2z8u0ynmj9setmh20".to_string();
        let output=||vec![p2tr.single_output(Address::from_str("bcrt1pcfc3whp89g2va7zwndpzjgdlzr4x03754nln0n0lvnj6rupm6pjq8xkj0r").unwrap().script_pubkey())];
        let lock_func=||P2tr::single_create_tx();

        let signer_1=get_xonly_fn(1);
        let signer_2=get_xonly_fn(2);
        let unlock_func_1=p2tr.input_factory(&signer_1,Address::from_str(&target_addr).unwrap().script_pubkey());

        let unlock_func_2=p2tr.input_factory(&signer_2,Address::from_str(&target_addr).unwrap().script_pubkey());
        let regtest=RegtestRpc::new(& vec![target_addr])();
        let psbt_1=create_partially_signed_tx(output(), lock_func(), unlock_func_1)(&regtest);

        let psbt_2=create_partially_signed_tx(output(), lock_func(), unlock_func_2)(&regtest);
        let combined_psbt=merge_psbt(&secp,&psbt_1,&psbt_2);
        combined_psbt.finalize(&secp).unwrap();
        // dbg!(combined_psbt);
        
        
    // tap_sign.input_factory(bob_keypair: &'a KeyPair, internal_key: XOnlyPublicKey);

}