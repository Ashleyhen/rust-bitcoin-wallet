use std::{env, vec};

use btc_wallet::{
    input_data::{electrum_rpc::{ElectrumRpc, ElectrumCall}, test_rpc_call::{TestRpc, TestCall}},
    spending_path::{
        p2tr_key_path::{ P2TRVault}, p2tr_multisig_path::P2trMultisig, vault_adaptor::VaultAdapter, Vault,
    },
    wallet_methods::{BroadcastOp, ClientWallet, ClientWithSchema}, address_formats::p2tr_addr_fmt::P2TR,
};
// use taproot_multi_sig::WalletInfo;
pub mod btc_wallet;
pub mod wallet_test;

// pub mod client_node;
// pub mod taproot_multi_sig;
fn main() {
    env::set_var("RUST_BACKTRACE", "full");
    test_transaction();
    // wallet_test::printOutTest();
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

    // let address_list = vec![to_addr.to_string(), tr_3.to_string()];
    // let aggregate = schema.aggregate(address_list);

    // let schema = P2TR::new(Some(seed.to_string()), 0, 3);
    let p2tr=P2TR::new( Some(seed.to_string()), 0, 3);
    let p2tr_vault= P2TRVault::new(&p2tr, 2000, &tr[3]);
    ;
    let client_with_schema = ClientWithSchema::new(&p2tr,ElectrumCall::new(&p2tr));
    client_with_schema.print_balance();
    let psbt = client_with_schema.submit_psbt(&p2tr_vault, BroadcastOp::Finalize);

    // let tr_script=P2TR::new( Some(seed.to_string()), 0, 4);
    // let script_vault = P2trMultisig::new(
    //     &tr_script,
    //     tr[3..].to_vec(),
    //     None,
    // );

    // let adapter = VaultAdapter::new(&p2tr_vault, &script_vault);
    // let client_with_schema_2 = ClientWithSchema::new(&tr_script, TestCall::new(&tr_script,&psbt));
    // let psbt=client_with_schema_2.submit_tx(&adapter);

}
// seed, vec<derivation path>
// p2wpkh 8
// tr 1
