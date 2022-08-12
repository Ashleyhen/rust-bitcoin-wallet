use std::collections::BTreeMap;
use std::{ops::Add, str::FromStr};

use bincode::config::LittleEndian;
use bitcoin::bech32::FromBase32;
use bitcoin::hashes::hex;
use bitcoin::psbt::{Input, Output, PartiallySignedTransaction, TapTree};
use bitcoin::secp256k1::{Message, Parity, Signature};
use bitcoin::util::taproot::LeafVersion::TapScript;
use bitcoin::util::taproot::{ControlBlock, LeafVersion};
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
use bitcoin::{Network, SchnorrSig, XOnlyPublicKey};
use bitcoin_hashes::hex::ToHex;
use miniscript::psbt::{PsbtExt, PsbtInputSatisfier};
use miniscript::ToPublicKey;

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
    //    16e1ae70ff0fa102905d4af297f6912bda6cce19
    let preimage_hash = bitcoin_hashes::sha256::Hash::hash(&preimage);

    println!("alice public key {}", alice.public_key());
    println!("bob public key {}", bob.public_key());
    println!("internal public key {}", internal.public_key());

    println!("preimage {}", preimage_hash.to_string());

    let alice_script = Script::from_hex(
        "029000b275209997a497d964fc1a62885b05a51166a65a90df00492c8d7cf61d6accf54803beac",
    )
    .unwrap();

    dbg!(alice_script.clone());

    let bob_script = Builder::new()
        .push_opcode(all::OP_SHA256)
        .push_slice(&preimage_hash)
        .push_opcode(all::OP_EQUALVERIFY)
        .push_x_only_key(&bob.public_key())
        .push_opcode(all::OP_CHECKSIG)
        .into_script();

    let alice_leaf = TapLeafHash::from_script(&alice_script, TapScript);
    let bob_leaf = TapLeafHash::from_script(&bob_script, TapScript);

    let alice_branch = TapBranchHash::from_inner(alice_leaf.into_inner());
    let bob_branch = TapBranchHash::from_inner(bob_leaf.into_inner());

    let branch = TapBranchHash::from_node_hashes(
        sha256::Hash::from_inner(alice_branch.into_inner()),
        sha256::Hash::from_inner(bob_branch.into_inner()),
    );

    let builder =
        TaprootBuilder::with_huffman_tree(vec![(1, bob_script.clone()), (1, alice_script.clone())])
            .unwrap();

    let tap_tree = TapTree::from_builder(builder).unwrap();
    let tap_info = tap_tree
        .into_builder()
        .finalize(&secp, internal.public_key())
        .unwrap();

    let tweak_key_pair = internal.tap_tweak(&secp, Some(branch)).into_inner();
    dbg!(tweak_key_pair.public_key());
    dbg!(tap_info.output_key());
    let address = Address::p2tr(
        &secp,
        internal.public_key(),
        Some(branch),
        bitcoin::Network::Regtest,
    );

    let tx = unsigned_tx();

    let sighash_sig = SighashCache::new(&mut tx.clone())
        .taproot_script_spend_signature_hash(
            0,
            &Prevouts::All(&vec![tx_input().output[0].clone()]),
            bob_leaf,
            SchnorrSighashType::Default,
        )
        .unwrap();

    let key_sig = SighashCache::new(&mut tx.clone())
        .taproot_key_spend_signature_hash(
            0,
            &Prevouts::All(&vec![tx_input().output[0].clone()]),
            SchnorrSighashType::Default,
        )
        .unwrap();
    println!("key signing sighash{}", key_sig);
    println!("key script sighash{}", sighash_sig);

    let sig = secp.sign_schnorr(
        &Message::from_slice(&sighash_sig).unwrap(),
        &bob.tap_tweak(&secp, Some(branch)).into_inner(),
    );

    let actual_control = ControlBlock {
        leaf_version: LeafVersion::TapScript,
        output_key_parity: Parity::Odd,
        internal_key: internal.public_key(),
        merkle_branch: TaprootMerkleBranch::from_slice(&alice_leaf).unwrap(),
    };

    let res =
        actual_control.verify_taproot_commitment(&secp, tweak_key_pair.public_key(), &bob_script);

    dbg!(res);

    let mut input = Input::default();
    // input.tap_scripts
    let mut b_tree_map = BTreeMap::<ControlBlock, (Script, LeafVersion)>::default();
    b_tree_map.insert(
        actual_control.clone(),
        (bob_script.clone(), LeafVersion::TapScript),
    );

    input.tap_scripts = b_tree_map;
    input.tap_internal_key = Some(internal.public_key());

    input.witness_utxo = Some(tx_input().output[0].clone());
    input.tap_merkle_root = Some(branch);

    let pst = PartiallySignedTransaction {
        unsigned_tx: unsigned_tx(),
        version: 2,
        xpub: BTreeMap::default(),
        proprietary: BTreeMap::default(),
        unknown: BTreeMap::default(),
        inputs: vec![input],
        outputs: vec![],
    };
    // dbg!(unsigned_tx());
    let schnorr_sig = SchnorrSig {
        sig,
        hash_ty: SchnorrSighashType::Default,
    };
    // sig
    println!("witness {}", sig);
    println!("Input preimage {}", preimage.to_hex());
    println!("script {}", bob_script.to_hex());
    println!("control block {:#?}", actual_control.serialize().to_hex());
    let wit = Witness::from_vec(vec![
        schnorr_sig.to_vec(),
        preimage,
        bob_script.to_bytes(),
        actual_control.serialize(),
    ]);
    dbg!(wit);
}
pub fn input_tx() -> Transaction {
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
pub fn unsigned_tx() -> Transaction {
    return Transaction::deserialize(&Vec::from_hex("020000000171f2f89c07c3b58c7b0cf3654ba049d28bbcc76b7298f41c17e7b1a3149040ec0000000000ffffffff01905f010000000000160014ceb2d28afdcad1ae0fc2cf81cb929ba29e83468200000000").unwrap()).unwrap();
}

pub fn tx_input() -> Transaction {
    return Transaction::deserialize(&Vec::from_hex("020000000001010aa633878f200c80fc8ec88f13f746e5870be7373ad5d78d22e14a402d6c6fc20000000000feffffff02a086010000000000225120a5ba0871796eb49fb4caa6bf78e675b9455e2d66e751676420f8381d5dda8951c759f405000000001600147bf84e78c81b9fed7a47b9251d95b13d6ebac14102473044022017de23798d7a01946744421fbb79a48556da809a9ffdb729f6e5983051480991022052460a5082749422804ad2a25e6f8335d5cf31f69799cece4a1ccc0256d5010701210257e0052b0ec6736ee13392940b7932571ce91659f71e899210b8daaf6f17027500000000").unwrap()).unwrap();
}

pub fn finalized() -> Transaction {
    return Transaction::deserialize(&Vec::from_hex("0200000000010171f2f89c07c3b58c7b0cf3654ba049d28bbcc76b7298f41c17e7b1a3149040ec0000000000ffffffff01905f010000000000160014ceb2d28afdcad1ae0fc2cf81cb929ba29e834682044054d5ee309be92f531d62449d8ef82b216f1e5b6229aaef918a78c26ce6dd66d57c523202b4650302667723f63dd5a87b2370ada51e08de0eccb27a80450ff9bf20107661134f21fc7c02223d50ab9eb3600bc3ffc3712423a1e47bb1f9a9dbf55f45a8206c60f404f8167a38fc70eaf8aa17ac351023bef86bcb9d1086a19afe95bd533388204edfcf9dfe6c0b5c83d1ab3f78d1b39a46ebac6798e08e19761f5ed89ec83c10ac41c1f30544d6009c8d8d94f5d030b2e844b1a3ca036255161c479db1cca5b374dd1cc81451874bd9ebd4b6fd4bba1f84cdfb533c532365d22a0a702205ff658b17c900000000").unwrap()).unwrap();
}
