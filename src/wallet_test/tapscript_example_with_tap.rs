use std::{ops::Add, str::FromStr};

use bincode::config::LittleEndian;
use bitcoin::bech32::FromBase32;
use bitcoin::hashes::hex;
use bitcoin::util::taproot::{ControlBlock, LeafVersion};
use bitcoin::{SchnorrSig, Network, XOnlyPublicKey};
use bitcoin::secp256k1::{Message, Signature, Parity};
use bitcoin::util::taproot::LeafVersion::TapScript;
use bitcoin::{
    blockdata::{opcodes::all, script::Builder},
    hashes::{hex::FromHex, sha256, Hash},
    psbt::serialize::Deserialize,
    schnorr::{TapTweak, TweakedKeyPair, TweakedPublicKey, UntweakedKeyPair},
    secp256k1::{Secp256k1, SecretKey},
    util::{
        bip32::ExtendedPrivKey,
        sighash::{Prevouts, SighashCache},
        taproot::{
            NodeInfo, TapBranchHash, TapBranchTag, TapLeafHash, TapSighashHash, TaprootBuilder,
            TaprootMerkleBranch, TaprootSpendInfo,
        },
    },
    Address, KeyPair, OutPoint, PrivateKey, SchnorrSighashType, Script, Transaction, TxIn, TxOut,
    Txid, Witness,
};
use bitcoin_hashes::hex::ToHex;

pub fn Test() {
    use bitcoin_hashes::Hash;
    let secp = Secp256k1::new();
    let alice_private =
        SecretKey::from_str("2bd806c97f0e00af1a1fc3328fa763a9269723c8db8fac4f93af71db186d6e90")
            .unwrap();
    let bob_private =
        SecretKey::from_str("81b637d8fcd2c6da6359e6963113a1170de795e4b725b84d1e0b4cfd9ec58ce9")
            .unwrap();
    let internal_private =
        SecretKey::from_str("1229101a0fcf2104e8808dab35661134aa5903867d44deb73ce1c7e4eb925be8")
            .unwrap();

    let alice = KeyPair::from_secret_key(&secp, alice_private);
    let bob = KeyPair::from_secret_key(&secp, bob_private);
    let internal = KeyPair::from_secret_key(&secp, internal_private);

    let preimage =
        Vec::from_hex("107661134f21fc7c02223d50ab9eb3600bc3ffc3712423a1e47bb1f9a9dbf55f").unwrap();
    let preimage_hash = bitcoin_hashes::sha256::Hash::hash(&preimage);

    println!("alice public key {}", alice.public_key());
    println!("bob public key {}", bob.public_key());
    println!("internal public key {}", internal.public_key());

    println!("preimage {}", preimage_hash.to_string());

    let script_alice = Script::from_hex(
        "029000b275209997a497d964fc1a62885b05a51166a65a90df00492c8d7cf61d6accf54803beac",
    )
    .unwrap();

    let bob_script = Builder::new()
        .push_opcode(all::OP_SHA256)
        .push_slice(&preimage_hash)
        .push_opcode(all::OP_EQUALVERIFY)
        .push_x_only_key(&bob.public_key())
        .push_opcode(all::OP_CHECKSIG)
        .into_script();
    let alice_leaf = TapLeafHash::from_script(&script_alice, TapScript);
    let bob_leaf = TapLeafHash::from_script(&bob_script, TapScript);

    let alice_branch = TapBranchHash::from_inner(alice_leaf.into_inner());
    let bob_branch = TapBranchHash::from_inner(bob_leaf.into_inner());
    let branch = TapBranchHash::from_node_hashes(
        sha256::Hash::from_inner(alice_branch.into_inner()),
        sha256::Hash::from_inner(bob_branch.into_inner()),
    );

    let spending = internal.tap_tweak(&secp, Some(branch)).into_inner();

    let address = Address::p2tr(&secp,internal.public_key(),Some(branch) ,bitcoin::Network::Regtest);

    dbg!(address.to_string());
	
    dbg!(spending.display_secret() );

    let tx=tx_as_hash();


// Key signing

    let sighash_key = SighashCache::new(&mut tx.clone())
        .taproot_key_spend_signature_hash(
            0,
            &Prevouts::All(&tx.output),
            SchnorrSighashType::Default,
        )
        .unwrap();

		let msg=Message::from_slice(&sighash_key).unwrap();

		let signature=secp.sign_schnorr(&msg,&internal);

	let shnorr_sig=SchnorrSig{ sig: signature, hash_ty:SchnorrSighashType::Default };
	
// script signing 
;
let sighash_sig=SighashCache::new(&mut tx.clone()).taproot_script_spend_signature_hash(0, &Prevouts::All(&tx.output), bob_leaf, SchnorrSighashType::Default).unwrap();



let actual_control =ControlBlock{ leaf_version: LeafVersion::TapScript, output_key_parity: Parity::Odd, internal_key: internal.public_key(), merkle_branch:TaprootMerkleBranch::from_slice(&alice_leaf).unwrap() };

let expected_control= ControlBlock::from_slice(&Vec::from_hex("c1f30544d6009c8d8d94f5d030b2e844b1a3ca036255161c479db1cca5b374dd1cc81451874bd9ebd4b6fd4bba1f84cdfb533c532365d22a0a702205ff658b17c9").unwrap()).unwrap();

dbg!(expected_control);
dbg!(actual_control.clone());


let res=actual_control.verify_taproot_commitment(&secp, spending.public_key(), &bob_script);
dbg!(internal.public_key().to_hex());

dbg!(bob_script.to_hex());
dbg!(res);
}

pub fn input_tx()->Transaction{
    return Transaction {
        version: 2,
        lock_time: 0,
        input: vec![
			TxIn {
				previous_output: OutPoint {
					txid: Txid::from_str(
						"ec409014a3b1e7171cf498726bc7bc8bd249a04b65f30c7b8cb5c3079cf8f271",
					)
					.unwrap(),
					vout: 0,
				},
				script_sig: Script::new(),
				sequence: 4294967294,
				witness:Witness::from_vec(
					vec![
						Vec::from_hex("3044022017de23798d7a01946744421fbb79a48556da809a9ffdb729f6e5983051480991022052460a5082749422804ad2a25e6f8335d5cf31f69799cece4a1ccc0256d5010701").unwrap(),
						Vec::from_hex("0257e0052b0ec6736ee13392940b7932571ce91659f71e899210b8daaf6f170275").unwrap()
					]),
				}],

        output: vec![
            TxOut {
                value: 100000,
                script_pubkey: Address::from_str("bcrt1p5kaqsuted66fldx256lh3en4h9z4uttxuagkwepqlqup6hw639gsm28t6c")
                .unwrap()
                .script_pubkey(),
            },
            TxOut {
                value: 99899847,
                script_pubkey: Address::from_str("bcrt1q00uyu7xgrw0767j8hyj3m9d384ht4s2p3058pr")
                    .unwrap()
                    .script_pubkey(),
            },
        ],
    };
}
pub fn tx_as_hash()->Transaction{
    return Transaction::deserialize(&Vec::from_hex("020000000171f2f89c07c3b58c7b0cf3654ba049d28bbcc76b7298f41c17e7b1a3149040ec0000000000ffffffff01905f010000000000160014ceb2d28afdcad1ae0fc2cf81cb929ba29e83468200000000").unwrap()).unwrap();
}
pub fn tx_as_input_hash()->Transaction{
    return Transaction::deserialize(&Vec::from_hex("020000000001010aa633878f200c80fc8ec88f13f746e5870be7373ad5d78d22e14a402d6c6fc20000000000feffffff02a086010000000000225120a5ba0871796eb49fb4caa6bf78e675b9455e2d66e751676420f8381d5dda8951c759f405000000001600147bf84e78c81b9fed7a47b9251d95b13d6ebac14102473044022017de23798d7a01946744421fbb79a48556da809a9ffdb729f6e5983051480991022052460a5082749422804ad2a25e6f8335d5cf31f69799cece4a1ccc0256d5010701210257e0052b0ec6736ee13392940b7932571ce91659f71e899210b8daaf6f17027500000000").unwrap()).unwrap();
}