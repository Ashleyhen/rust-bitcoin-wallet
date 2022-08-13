use bitcoin::{
    blockdata::{opcodes::all, script::Builder},
    hashes::hex::FromHex,
    psbt::Output,
    secp256k1::SecretKey,
    Script, Transaction, TxIn, TxOut,
};
use bitcoin_hashes::Hash;

use crate::btc_wallet::{
    address_formats::{p2tr_addr_fmt::P2TR, AddressSchema},
    constants::TIP,
    script_services::{
        api::{LockFn, UnlockFn},
        input_service::InputService,
        output_service::OutputService,
    },
};
pub fn bob_scripts(tr: &P2TR) -> Script {
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

    return bob_script;
}

pub fn alice_script() -> Script {
    let script = Script::from_hex(
        "029000b275209997a497d964fc1a62885b05a51166a65a90df00492c8d7cf61d6accf54803beac",
    )
    .unwrap();
    return script;
}

pub fn output_factory<'a>(
    alice: &'a OutputService,
    bob: &'a OutputService,
    secret: SecretKey,
) -> Vec<LockFn<'a>> {
    let bob_script = bob_scripts(&bob.0);
    let alice_script = alice_script();
    let combined_script = vec![(1, bob_script.clone()), (1, alice_script.clone())];
    return vec![
        bob.new_tap_internal_key(secret),
        alice.insert_tap_key_origin(vec![(1, alice_script)]),
        bob.insert_tap_key_origin(vec![(1, bob_script)]),
        P2TR::insert_tap_tree(combined_script),
        bob.insert_witness(),
    ];
}

pub fn input_factory<'a>(
    bob: &'a InputService,
) -> Box<dyn Fn(Vec<Transaction>, Transaction) -> Vec<UnlockFn<'a>> + 'a> {
    let bob_script = bob_scripts(&bob.0);
    return Box::new(
        move |previous_list: Vec<Transaction>, current_tx: Transaction| {
            let mut unlock_vec: Vec<UnlockFn> = vec![];
            for (size, prev) in previous_list.iter().enumerate() {
                unlock_vec.push(bob.insert_givens());
                unlock_vec.push(bob.insert_control_block(bob_script.clone()));
                unlock_vec.push(bob.sign_tapleaf(current_tx.clone(), prev.clone().output, size));
            }
            return unlock_vec;
        },
    );
}

pub fn create_tx(amount: u64) -> Box<dyn Fn(Vec<Output>, Vec<TxIn>, u64) -> Transaction> {
    return Box::new(move |outputs: Vec<Output>, tx_in: Vec<TxIn>, total: u64| {
        let receiver = 0;
        let change = 1;
        let tx_out_list = || {
            let tx_out = TxOut {
                value: amount,
                script_pubkey: outputs[receiver].clone().witness_script.unwrap(),
            };

            if (total - amount) > TIP {
                return vec![
                    tx_out,
                    TxOut {
                        value: total - amount,
                        script_pubkey: outputs[change].clone().witness_script.unwrap(),
                    },
                ];
            }
            return vec![tx_out];
        };
        return Transaction {
            version: 2,
            lock_time: 0,
            input: tx_in,
            output: tx_out_list(),
        };
    });
}
