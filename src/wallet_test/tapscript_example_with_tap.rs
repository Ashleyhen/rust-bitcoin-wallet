use std::str::FromStr;

use bitcoin::{util::{bip32::ExtendedPrivKey, taproot::{TapLeafHash, TaprootBuilder, TaprootMerkleBranch, NodeInfo, TaprootSpendInfo, TapBranchTag, TapBranchHash, TapSighashHash}, sighash::{SighashCache, Prevouts}}, PrivateKey, secp256k1::{SecretKey, Secp256k1}, KeyPair, hashes::{hex::FromHex, sha256, Hash}, Script, blockdata::{script::Builder, opcodes::all}, Address, schnorr::{TweakedKeyPair, TapTweak, TweakedPublicKey, UntweakedKeyPair}, Transaction, psbt::serialize::Deserialize, SchnorrSighashType};
use bitcoin::util::taproot::LeafVersion::TapScript;

pub fn Test(){
use bitcoin_hashes::Hash;
	let secp=Secp256k1::new();
	let alice_private=SecretKey::from_str("2bd806c97f0e00af1a1fc3328fa763a9269723c8db8fac4f93af71db186d6e90").unwrap();
	let bob_private=SecretKey::from_str("81b637d8fcd2c6da6359e6963113a1170de795e4b725b84d1e0b4cfd9ec58ce9").unwrap();
	let internal_private=SecretKey::from_str("1229101a0fcf2104e8808dab35661134aa5903867d44deb73ce1c7e4eb925be8").unwrap();

	let alice=KeyPair::from_secret_key(&secp, alice_private);
	let bob=KeyPair::from_secret_key(&secp,bob_private);
	let internal=KeyPair::from_secret_key(&secp, internal_private);


	let preimage=Vec::from_hex("107661134f21fc7c02223d50ab9eb3600bc3ffc3712423a1e47bb1f9a9dbf55f").unwrap();
	let preimage_hash=bitcoin_hashes::sha256::Hash::hash(&preimage);

	println!("alice public key {}",alice.public_key());
	println!("bob public key {}",bob.public_key());
	println!("internal public key {}",internal.public_key());

	println!("preimage {}",preimage_hash.to_string());


	let script_alice=Script::from_hex("029000b275209997a497d964fc1a62885b05a51166a65a90df00492c8d7cf61d6accf54803beac").unwrap();
	
	let  script_bob=Builder::new()
	.push_opcode(all::OP_SHA256)
	.push_slice(&preimage_hash)
	.push_opcode(all::OP_EQUALVERIFY)
	.push_x_only_key(&bob.public_key())
	.push_opcode(all::OP_CHECKSIG)
	.into_script();


	let alice_leaf=TapLeafHash::from_script(&script_alice,TapScript);
	let bob_leaf=TapLeafHash::from_script(&script_bob,TapScript);

	let alice_branch=TapBranchHash::from_inner(alice_leaf.into_inner());
	let bob_branch=TapBranchHash::from_inner(bob_leaf.into_inner());
	 let branch =TapBranchHash::from_node_hashes(
                sha256::Hash::from_inner(alice_branch.into_inner()),
                sha256::Hash::from_inner(bob_branch.into_inner())
                );

			let spending=TaprootSpendInfo::new_key_spend(&secp,internal.public_key(), Some(branch));
			let address = Address::p2tr_tweaked(spending.output_key(), bitcoin::Network::Regtest);

				dbg!(address.to_string());


	let mut tx=Transaction::deserialize(&Vec::from_hex("020000000171f2f89c07c3b58c7b0cf3654ba049d28bbcc76b7298f41c17e7b1a3149040ec0000000000ffffffff01905f010000000000160014ceb2d28afdcad1ae0fc2cf81cb929ba29e83468200000000")
	.unwrap()).unwrap();
 let sighash =SighashCache::new(&mut tx.clone())
            .taproot_key_spend_signature_hash(
                0,
                &Prevouts::All(&tx.output),
                SchnorrSighashType::AllPlusAnyoneCanPay,
            )
            .unwrap();

	dbg!(tx);
	/* 
	bcrt1p5kaqsuted66fldx256lh3en4h9z4uttxuagkwepqlqup6hw639gsm28t6c
bcrt1p5kaqsuted66fldx256lh3en4h9z4uttxuagkwepqlqup6hw639gsm28t6c
	let tap_pub=TaprootSpendInfo::from_node_info(&secp, internal.public_key(),node_info).output_key();
	// let finalize=builder.finalize(&secp, internal.public_key()).unwrap();
	dbg!(tap_pub);
	TaprootMerkleBranch::from_slice(&alice_leaf).unwrap();
	let taproot_spending=TaprootBuilder::new()
	.add_leaf(0, script_alice).unwrap().add_leaf(1,script_bob).unwrap().finalize(&secp, internal.public_key()).unwrap();
	let addr=Address::p2tr_tweaked(taproot_spending.output_key(), bitcoin::Network::Regtest);
	dbg!(addr.to_string());
// leaf.hash(state)),bob_leaf);
	// lice_leaf);
	println!("bob leaf {}", bob_leaf);
	println!("aliceleaf {}", alice_leaf);

		*/
		// 
	

		// 
	
}