use std::{env, str::FromStr};

use bitcoin::{
    blockdata::{opcodes::all, script::Builder},
    hashes::hex::FromHex,
    util::taproot::ControlBlock,
    Script, KeyPair, secp256k1::SecretKey,
};
use bitcoin_hashes::Hash;
use btc_wallet::{
    address_formats::{p2tr_addr_fmt::P2TR, AddressSchema},
    input_data::{
        electrum_rpc::{ElectrumCall, ElectrumRpc},
        reuse_rpc_call::{ReUseCall, TestRpc}, tapscript_ex_input::TapscriptExInput,
    },
    spending_path::{
        mutlisig_path::MultiSigPath, p2tr_key_path::P2TRVault, p2tr_multisig_path::P2trMultisig,
        vault_adaptor::VaultAdapter, Vault,
    },
    wallet_methods::{BroadcastOp, ClientWallet, ClientWithSchema},
};
use either::Either;
use wallet_test::{tapscript_example_with_tap::Test, wallet_test_vector_traits::WalletTestVectors};
// use taproot_multi_sig::WalletInfo;
pub mod btc_wallet;
pub mod wallet_test;

// pub mod client_node;
// pub mod taproot_multi_sig;
fn main() {
    env::set_var("RUST_BACKTRACE", "full");
    // let wallet_test_vectors = WalletTestVectors::load_test();
    // wallet_test_vectors.test();
    Test();

    control_block_test();
    // wallet_test_vectors.test();
}

pub fn control_block_test() {
    let alice_seed = 0;
    let bob_seed = 1;
    let internal_seed = 2;

    let seeds = vec![
        "2bd806c97f0e00af1a1fc3328fa763a9269723c8db8fac4f93af71db186d6e90", //alice
        "81b637d8fcd2c6da6359e6963113a1170de795e4b725b84d1e0b4cfd9ec58ce9", //bob
        "1229101a0fcf2104e8808dab35661134aa5903867d44deb73ce1c7e4eb925be8", //internal
    ];

    let alice_addr = P2TR::new(Some(seeds[alice_seed].to_string()), 0, 0);
    let bob_addr = P2TR::new(Some(seeds[bob_seed].to_string()), 0, 0);

    let alice_script = Script::from_hex(
        "029000b275209997a497d964fc1a62885b05a51166a65a90df00492c8d7cf61d6accf54803beac",
    )
    .unwrap();
    let preimage =
        Vec::from_hex("107661134f21fc7c02223d50ab9eb3600bc3ffc3712423a1e47bb1f9a9dbf55f").unwrap();
    let preimage_hash = bitcoin_hashes::sha256::Hash::hash(&preimage);

    let pair=KeyPair::from_secret_key(&bob_addr.to_wallet().secp, SecretKey::from_str(seeds[bob_seed]).unwrap());

    let bob_script=Script::from_hex("a8206c60f404f8167a38fc70eaf8aa17ac351023bef86bcb9d1086a19afe95bd533388204edfcf9dfe6c0b5c83d1ab3f78d1b39a46ebac6798e08e19761f5ed89ec83c10ac").unwrap();
    // let bob_script = Builder::new()
    //     .push_opcode(all::OP_SHA256)
    //     .push_slice(&preimage_hash)
    //     .push_opcode(all::OP_EQUALVERIFY)
    //     .push_x_only_key(&pair.public_key())
    //     .push_opcode(all::OP_CHECKSIG)
    //     .into_script();


//     OP_SHA256
// 6c60f404f8167a38fc70eaf8aa17ac351023bef86bcb9d1086a19afe95bd5333
// OP_EQUALVERIFY
// 4edfcf9dfe6c0b5c83d1ab3f78d1b39a46ebac6798e08e19761f5ed89ec83c10
// OP_CHECKSIG
    let internal_seed = seeds[2].to_string();
    let alice_schema = ClientWithSchema::new(&alice_addr, TapscriptExInput::new());
    let alice_vault = MultiSigPath::new(&alice_addr, None, Some(&internal_seed), &alice_script);

    let alice_tx_part = alice_schema.submit_psbt(&alice_vault, BroadcastOp::None);

    let bob_schema = ClientWithSchema::new(&bob_addr, TapscriptExInput::new());

    let bob_vault = MultiSigPath::new(
        &bob_addr,
        Some(&alice_tx_part),
        Some(&internal_seed),
        &bob_script,
    );
    let bob_tx_part = bob_schema.submit_psbt(&bob_vault, BroadcastOp::None);

    let p2tr=P2TR::new(Some("1d454c6ab705f999d97e6465300a79a9595fb5ae1186ae20e33e12bea606c094".to_string()), 0, 0);
    let tr_vault = P2TRVault::new(&p2tr, 2000, &"tb1puma0fas8dgukcvhm8ewsganj08edgnm6ejyde3ev5lvxv4h7wqvqpjslxz".to_string());
    let adapter=VaultAdapter::new(&tr_vault,&bob_vault);

    let spender_schema = ClientWithSchema::new(&p2tr, ReUseCall::<P2TR>::new(None,&bob_tx_part));


    let bob_tx_part = spender_schema.submit_psbt(&adapter, BroadcastOp::Finalize);
    // let result_vault = P2TRVault::new(&addr[reciever], 1000, &tr[alice]);
    // let tr_to_alice = VaultAdapter::new(&alice_vault, &bob_vault);
}

fn test_transaction() {
    // person 1
    let seed = "1d454c6ab705f999d97e6465300a79a9595fb5ae1186ae20e33e12bea606c094";
    let tr = vec![
        "tb1puma0fas8dgukcvhm8ewsganj08edgnm6ejyde3ev5lvxv4h7wqvqpjslxz".to_string(),
        "tb1phtgnyv6qj4n6kqkmm2uzg630vz2tmgv4kchdp44j7my6qre4qdys6hchvx".to_string(),
        "tb1p95xjusgkgh2zqhyr5q9hzwv607yc5dncnsastm9xygmmuu4xrcqs53468m".to_string(),
        "tb1pz6egnzpq0h92zjkv23vdt4gwy8thd4t0t66megj20cr32m64ds4qv2kcal".to_string(),
        "tb1p69eefuuvaalsdljjyqntnrrtc4yzpc038ujm3ppze8g6ljepskks2zzffj".to_string(),
    ];

    let addr: Vec<P2TR> = (0..4)
        .map(|c| P2TR::new(Some(seed.to_string()), 0, c))
        .collect();

    let tr_vault = P2TRVault::new(&addr[3], 2000, &tr[0]);

    // let client_with_schema = ClientWithSchema::new(&addr[3], ElectrumCall::new(&addr[3]));
    // client_with_schema.print_balance();
    // let psbt = client_with_schema.submit_psbt(&three_vault, BroadcastOp::Finalize);

    let sender = 3;
    let alice = 0;
    let bob = 1;
    let reciever = 2;

    let alice_script = addr[sender].alice_script(addr[sender].get_ext_pub_key().to_x_only_pub());
    let alice_vault = MultiSigPath::new(&addr[alice], None, None, &alice_script);
    let tr_to_alice = VaultAdapter::new(&alice_vault, &tr_vault);

    let alice_wallet = ClientWithSchema::new(&addr[alice], ElectrumCall::new(&addr[sender]));
    let alice_psbt = alice_wallet.submit_psbt(&tr_to_alice, BroadcastOp::None);

    // let bob_script=P2TR::bob_script(&p2tr.get_ext_pub_key().to_x_only_pub());

    let bob_script = P2TR::bob_script(&addr[bob].get_ext_pub_key().to_x_only_pub());

    let a_and_b_vault = MultiSigPath::new(&addr[bob], Some(&alice_psbt), None, &bob_script);

    let bob_wallet =
        ClientWithSchema::new(&addr[bob], ReUseCall::new(Some(&addr[sender]), &alice_psbt));
    let mutisig_signed_tx = bob_wallet.submit_psbt(&a_and_b_vault, BroadcastOp::None);

    let result_vault = P2TRVault::new(&addr[reciever], 1000, &tr[alice]);

    let preimage = Builder::new()
        .push_slice(
            &Vec::from_hex("107661134f21fc7c02223d50ab9eb3600bc3ffc3712423a1e47bb1f9a9dbf55f")
                .unwrap(),
        )
        .into_script();
    let bob_vault = MultiSigPath::new(&addr[bob], Some(&mutisig_signed_tx), None, &bob_script);
    let final_adapter = VaultAdapter::new(&result_vault, &bob_vault);
    let signer_schema_1 = ClientWithSchema::new(
        &addr[sender],
        ReUseCall::<P2TR>::new(None, &mutisig_signed_tx),
    );
    let bob_signed_tx = signer_schema_1.submit_psbt(&final_adapter, BroadcastOp::Finalize);

    // ControlBlock
}
// seed, vec<derivation path>
// p2wpkh 8
// tr 1
