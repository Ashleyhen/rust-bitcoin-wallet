use std::env;

use bitcoin::Address;
use traproot_bdk::{
    connect_lightning,
    lnrpc::{
        AddressType, ConnectPeerRequest, GetInfoRequest, LightningAddress, ListPeersRequest,
        NewAddressRequest, NodeInfoRequest, OpenChannelRequest,
    },
};

use crate::{
    bitcoin_wallet::{
        constants::{MINE, NETWORK, SEED},
        input_data::regtest_call::RegtestCall,
    },
    simple_wallet::{
        freelancer::{bisq, bisq_script::BisqScript, bisq_key::BisqKey},
        p2tr_key::P2TR,
        p2tr_script::{self, bob_scripts, create_address, preimage, P2TRS},
        p2wpkh::P2WPKH,
        p2wsh::P2WSH,
        single_output, single_output_with_value, SendToImpl, Wallet,
    },
};
pub mod bitcoin_wallet;
pub mod lighting;
pub mod simple_wallet;
// rm -rf ~/.docker/volumes/lightningd_data/ && rm -rf ~/.docker/volumes/lnd_data/
#[tokio::main]
async fn main() {
    env::set_var("RUST_BACKTRACE", "full");
}

#[test]
fn test_tap_root_key_sig() {
    println!("Testing layer 1 pay to tap root with key signature");
    let client = RegtestCall::init(
        &vec!["bcrt1prnpxwf9tpjm4jll4ts72s2xscq66qxep6w9hf6sqnvwe9t4gvqasklfhyj"],
        "my_wallet",
        MINE,
    );


    P2TR::new(Some(SEED), &client).send(single_output());
}

#[test]
fn test_pay_2_witness_public_key_hash() {
    println!("Testing layer 1 pay to witness public key signature");
    let client = RegtestCall::init(
        &vec!["bcrt1qzvsdwjay5x69088n27h0qgu0tm4u6gwqgxna9d"],
        "my_wallet",
        MINE,
    );
    P2WPKH::new(Some(SEED), &client).send(single_output_with_value(
        "bcrt1pz7f9jke4mpa6gfwgcqn370ajpk8jz484yfyypy3mqwnqjtj9vhdqf0rrnp".to_owned(),
    ));
}

#[test]
fn test_pay_2_witness_script_hash() {
    println!("Testing layer 1 pay to witness script signature");

    let alice_seed = "1d454c6ab705f999d97e6465300a79a9595fb5ae1186ae20e33e12bea606c094";

    let bob_seed = "81b637d8fcd2c6da6359e6963113a1170de795e4b725b84d1e0b4cfd9ec58ce9";

    type P2wsh<'a> = P2WSH<'a, RegtestCall>;

    let alice_pub_key = P2wsh::seed_to_pubkey(&Some(alice_seed));

    let bob_pub_key = P2wsh::seed_to_pubkey(&Some(bob_seed));

    let pub_keys = vec![bob_pub_key, alice_pub_key];
    let target_address = P2wsh::multi_sig_address(&pub_keys);

    println!("target address {}", target_address.to_string());

    let client = RegtestCall::init(&vec![&target_address.to_string()], "my_wallet", MINE);
    let output = single_output();
    let alice_psbt = P2WSH::new(Some(alice_seed), &client).parital_sig(&pub_keys, None, &output);

    let bob = P2WSH::new(Some(bob_seed), &client);

    let bob_psbt = bob.parital_sig(&pub_keys, Some(alice_psbt), &output);

    bob.broadcasted(bob_psbt);
}

#[test]
fn test_pay_2_taproot_script() {
    println!("Testing layer 1 pay to witness public key signature");

    let alice_seed = "2bd806c97f0e00af1a1fc3328fa763a9269723c8db8fac4f93af71db186d6e90";

    let bob_seed = "81b637d8fcd2c6da6359e6963113a1170de795e4b725b84d1e0b4cfd9ec58ce9";

    let bob_image = "107661134f21fc7c02223d50ab9eb3600bc3ffc3712423a1e47bb1f9a9dbf55f";

    type P2trs<'a> = P2TRS<'a, RegtestCall>;

    let alice_xonly = p2tr_script::seed_to_xonly(&Some(alice_seed));

    let bob_xonly = p2tr_script::seed_to_xonly(&Some(bob_seed));

    let preimage = preimage(bob_image);

    let output = create_address(alice_xonly, bob_xonly, preimage);

    let address = Address::from_script(&output.clone().witness_script.unwrap(), NETWORK).unwrap();

    let client = RegtestCall::init(&vec![&address.to_string()], "my_wallet", MINE);

    let bob_wallet = P2trs::new(bob_seed, bob_image, &client);

    bob_wallet.sign(&output, single_output());
}


#[test]
fn bisq_with_tr_script() {
    println!("Testing layer 1 pay to witness public key signature");

    let secret_host = "2bd806c97f0e00af1a1fc3328fa763a9269723c8db8fac4f93af71db186d6e90";

    let secret_client = "81b637d8fcd2c6da6359e6963113a1170de795e4b725b84d1e0b4cfd9ec58ce9";

    let support_team = "107661134f21fc7c02223d50ab9eb3600bc3ffc3712423a1e47bb1f9a9dbf55f";

    let host_xonly = p2tr_script::seed_to_xonly(&Some(secret_host));

    let client_xonly = p2tr_script::seed_to_xonly(&Some(secret_client));

    let support_team_xonly = p2tr_script::seed_to_xonly(&Some(support_team));

    let output = bisq::create_address(&host_xonly, &client_xonly, &support_team_xonly);

    let address = Address::from_script(&output.clone().witness_script.unwrap(), NETWORK).unwrap();

    dbg!(address.to_string());

    let regtestcall = RegtestCall::init(&vec![&address.to_string()], "my_wallet", MINE);

    let host_wallet = bisq::Bisq::new(
        secret_host,
        &regtestcall,
        BisqScript {
            output: output.clone(),
            input: vec![],
        },
    );

    let host_psbt = host_wallet.sign(&output, None, single_output());

    let client_wallet = bisq::Bisq::new(
        secret_client,
        &regtestcall,
        BisqScript {
            output: output.clone(),
            input: host_psbt.inputs.clone(),
        },
    );
    let client_psbt = client_wallet.sign(&output, Some(host_psbt), single_output());

    client_wallet.finalize_script(client_psbt);
}

#[test]
fn bisq_with_tr_key() {
    println!("Testing layer 1 pay to witness public key signature");

    let secret_host = "2bd806c97f0e00af1a1fc3328fa763a9269723c8db8fac4f93af71db186d6e90";

    let secret_client = "81b637d8fcd2c6da6359e6963113a1170de795e4b725b84d1e0b4cfd9ec58ce9";

    let support_team = "107661134f21fc7c02223d50ab9eb3600bc3ffc3712423a1e47bb1f9a9dbf55f";

    let host_xonly = p2tr_script::seed_to_xonly(&Some(secret_host));

    let client_xonly = p2tr_script::seed_to_xonly(&Some(secret_client));

    let support_team_xonly = p2tr_script::seed_to_xonly(&Some(support_team));

    let output = bisq::create_address(&host_xonly, &client_xonly, &support_team_xonly);

    let address = Address::from_script(&output.clone().witness_script.unwrap(), NETWORK).unwrap();

    dbg!(address.to_string());

    let regtestcall = RegtestCall::init(&vec![&address.to_string()], "my_wallet", MINE);

    let support_team_wallet = bisq::Bisq::new(
        support_team,
        &regtestcall,
        BisqKey {
            output: output.clone(),
        },
    );

    let psbt = support_team_wallet.sign(&output, None, single_output());

    support_team_wallet.finalize_script(psbt);
}
