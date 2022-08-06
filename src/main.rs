use std::env;

use bitcoin::util::taproot::ControlBlock;
use btc_wallet::{
    address_formats::{p2tr_addr_fmt::P2TR, AddressSchema},
    input_data::{
        electrum_rpc::{ElectrumCall, ElectrumRpc},
        reuse_rpc_call::{ReUseCall, TestRpc},
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
    test_transaction();
    // let wallet_test_vectors = WalletTestVectors::load_test();
    // wallet_test_vectors.test();
    // Test()
    // wallet_test_vectors.test();
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

    let addr:Vec<P2TR>=(0..4).map(|c|P2TR::new(Some(seed.to_string()), 0, c)).collect();
    
    let tr_vault = P2TRVault::new(&addr[3], 2000, &tr[0]);

    // let client_with_schema = ClientWithSchema::new(&addr[3], ElectrumCall::new(&addr[3]));
    // client_with_schema.print_balance();
    // let psbt = client_with_schema.submit_psbt(&three_vault, BroadcastOp::Finalize);

    let sender=3;
    let alice=0;
    let bob=1;
    let reciever=2;
    
    let alice_script = addr[sender].alice_script(addr[sender].get_ext_pub_key().to_x_only_pub());
    let alice_vault = MultiSigPath::new(&addr[alice], None, &alice_script);
    let tr_to_alice = VaultAdapter::new(&alice_vault, &tr_vault);
    
    let alice_wallet = ClientWithSchema::new(&addr[alice], ElectrumCall::new(&addr[sender]));
    let alice_psbt = alice_wallet.submit_psbt(&tr_to_alice, BroadcastOp::None);

    // let bob_script=P2TR::bob_script(&p2tr.get_ext_pub_key().to_x_only_pub());

    let bob_script=P2TR::bob_script(&addr[bob].get_ext_pub_key().to_x_only_pub());

    let a_and_b_vault = MultiSigPath::new(&addr[bob],Some(&alice_psbt), &bob_script);

    let bob_wallet = ClientWithSchema::new(&addr[bob],
         ReUseCall::new(Some(&addr[sender]), &alice_psbt));
    let mutisig_signed_tx = bob_wallet.submit_psbt(&a_and_b_vault, BroadcastOp::None);

    let result_vault = P2TRVault::new(&addr[reciever], 1000, &tr[alice]);

    let bob_vault = MultiSigPath::new(&addr[bob], Some(&mutisig_signed_tx), &bob_script);
    let final_adapter = VaultAdapter::new(&result_vault,&bob_vault );
    let signer_schema_1 = ClientWithSchema::new(&addr[sender], ReUseCall::<P2TR>::new(None, &mutisig_signed_tx));
    let mutisig_signed_tx = signer_schema_1.submit_psbt(&final_adapter, BroadcastOp::Finalize);




    // ControlBlock
}
// seed, vec<derivation path>
// p2wpkh 8
// tr 1
