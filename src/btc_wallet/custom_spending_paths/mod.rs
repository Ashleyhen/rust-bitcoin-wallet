use bitcoin::{
    blockdata::{opcodes::all, script::Builder},
    hashes::hex::FromHex,
    psbt::Output,
    Address, Script,
};
use bitcoin_hashes::Hash;

use crate::btc_wallet::{constants::NETWORK, script_services::output_service::OutputService};

use super::address_formats::{p2tr_addr_fmt::P2TR, AddressSchema};

pub fn bob_scripts(tr: &P2TR) -> Vec<(u32, Script)> {
    let x_only = tr.get_ext_pub_key().to_x_only_pub();
    let preimage =
        Vec::from_hex("107661134f21fc7c02223d50ab9eb3600bc3ffc3712423a1e47bb1f9a9dbf55f").unwrap();
    let preimage_hash = bitcoin_hashes::sha256::Hash::hash(&preimage);
    let bob_script = Builder::new()
        .push_opcode(all::OP_SHA256)
        .push_slice(&preimage_hash)
        .push_opcode(all::OP_EQUALVERIFY)
        .push_x_only_key(&x_only)
        .push_opcode(all::OP_CHECKSIG)
        .into_script();

    return vec![(1, bob_script)];
}

pub fn alice_script() -> Vec<(u32, Script)> {
    let script = Script::from_hex(
        "029000b275209997a497d964fc1a62885b05a51166a65a90df00492c8d7cf61d6accf54803beac",
    )
    .unwrap();
    return vec![(1, script)];
}
pub fn config(bob_addr: P2TR, alice_addr: P2TR) -> Vec<Output> {
    let secret_key =
        alice_addr.new_shared_secret(vec![bob_addr.get_ext_pub_key().to_x_only_pub()].iter());
    let alice_script = alice_script();
    let bob_script = bob_scripts(&bob_addr);
    let combined_script = vec![bob_script.clone(), alice_script.clone()].concat();

    let bob_output_service = OutputService(bob_addr);
    let alice_output_service = OutputService(alice_addr);

    let mut func_list: Vec<Box<dyn for<'r> FnMut(&'r mut bitcoin::psbt::Output)>> = vec![
        bob_output_service.new_tap_internal_key(&secret_key),
        alice_output_service.insert_tap_key_origin(&alice_script),
        bob_output_service.insert_tap_key_origin(&bob_script),
        P2TR::insert_tap_tree(&combined_script),
        bob_output_service.insert_witness(),
    ];

    let mut output = Output::default();

    for output_func in &mut func_list {
        output_func(&mut output);
    }

    let addr = Address::from_script(&output.clone().witness_script.unwrap(), NETWORK).unwrap();
    dbg!(addr.to_string());
    return vec![output];
}
