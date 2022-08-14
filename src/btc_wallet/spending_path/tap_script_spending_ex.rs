use bitcoin::{
    blockdata::{opcodes::all, script::Builder},
    hashes::hex::FromHex,
    psbt::Output,
    secp256k1::{All, Secp256k1},
    KeyPair, Script, Transaction, TxIn, TxOut, XOnlyPublicKey,
};
use bitcoin_hashes::Hash;

use crate::{btc_wallet::{
    constants::{TIP},
    script_services::{
        api::{LockFn, UnlockFn},
        input_service::{insert_control_block, insert_givens, sign_tapleaf},
        output_service::{
            insert_tap_key_origin, insert_tap_tree, insert_witness, new_tap_internal_key,
        },
    },
}, wallet_test::tapscript_example_with_tap::unsigned_tx};

pub fn bob_scripts(x_only: &XOnlyPublicKey) -> Script {
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
    secp: &'a Secp256k1<All>,
    xinternal: XOnlyPublicKey,
    xalice: XOnlyPublicKey,
    xbob: XOnlyPublicKey,
) -> Vec<LockFn<'a>> {
    let bob_script = bob_scripts(&xbob);
    let alice_script = alice_script();
    let combined_script = vec![(1, bob_script.clone()), (1, alice_script.clone())];
    return vec![
        new_tap_internal_key(xinternal),
        insert_tap_key_origin(vec![(1, alice_script)], xalice),
        insert_tap_key_origin(vec![(1, bob_script)], xbob),
        insert_tap_tree(combined_script),
        insert_witness(&secp),
    ];
}

pub fn input_factory<'a>(
    secp: &'a Secp256k1<All>,
    keypair: &'a KeyPair,
) -> Box<dyn Fn(Vec<Transaction>, Transaction) -> Vec<UnlockFn<'a>> + 'a> {
    let x_only = keypair.public_key();
    let bob_script = bob_scripts(&x_only);
    return Box::new(
        move |previous_list: Vec<Transaction>, current_tx: Transaction| {
            let mut unlock_vec: Vec<UnlockFn> = vec![];
            for (size, prev) in previous_list.iter().enumerate() {
                unlock_vec.push(insert_givens());
                unlock_vec.push(insert_control_block(secp, x_only, bob_script.clone()));
                unlock_vec.push(sign_tapleaf(
                    secp,
                    &keypair,
                    current_tx.clone(),
                    prev.clone().output,
                    size,
                ));
            }
            return unlock_vec;
        },
    );
}

pub fn create_tx(total: u64) -> Box<dyn Fn(Vec<Output>, Vec<TxIn>, u64) -> Transaction> {
    return Box::new(move |outputs: Vec<Output>, tx_in: Vec<TxIn>, amount: u64| {
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

pub fn example_tx(_: u64) -> Box<dyn Fn(Vec<Output>, Vec<TxIn>, u64) -> Transaction> {
    return Box::new(move |_: Vec<Output>, _: Vec<TxIn>, _: u64| {
        return unsigned_tx();
    });
}