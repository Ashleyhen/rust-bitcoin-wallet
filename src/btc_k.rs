use std::{str::FromStr, ops::Add, sync::Arc, collections::BTreeMap};

use bdk::{ KeychainKind};
use bitcoin::{Network, secp256k1::{constants, Secp256k1, All, SecretKey, rand::rngs::OsRng, Message}, Script, util::{bip32::{DerivationPath, ChildNumber, ExtendedPrivKey, ExtendedPubKey}, sighash::{self, SighashCache}}, Address, PublicKey, TxIn, OutPoint, Witness, Transaction, TxOut, psbt::{Input, Output, PartiallySignedTransaction}, blockdata::{script::Builder, opcodes}, EcdsaSig};
use electrum_client::{Client, ElectrumApi};
use miniscript::{ToPublicKey, psbt::PsbtExt};

pub struct BtcK{
    secp:Secp256k1<All>,
    client:Client,
    seed:Seed
}
type Seed=[u8;constants::SECRET_KEY_SIZE];

const NETWORK: bitcoin::Network = Network::Testnet;
impl BtcK{
    pub fn new(secret_seed:Option<String>)->BtcK {
        BtcK{
            secp:Secp256k1::new(),
			client: Client::new("ssl://electrum.blockstream.info:60002").expect("client connection failed !!!"),
            seed: (match secret_seed{
			Some(secret)=> SecretKey::from_str(&secret).expect("failed to initalize seed, check if the seed is valid"),
			_ => SecretKey::new(&mut OsRng::new().expect("failed to create a random number generator !!! ") ) }).secret_bytes()
        }
    }
    
    fn get_ext_keys(seed:&Seed,derivation_p:&Option<DerivationPath>, secp:&Secp256k1<All>)->(ExtendedPrivKey,ExtendedPubKey){
        let ext_prv=match derivation_p{
            Some(dp) => ExtendedPrivKey::new_master(NETWORK, seed).unwrap().derive_priv(&secp,&dp),
            None => ExtendedPrivKey::new_master(NETWORK, seed),
        }.unwrap();
        return (ext_prv,ExtendedPubKey::from_priv(&secp,&ext_prv));
    }

    pub fn get_balance(&self){
        let (_,extended_pub)=BtcK::get_ext_keys( &self.seed,&Some(BtcK::derivation_path(Some(4),0)), &self.secp);
        BtcK::balance( &self.client, &extended_pub.public_key.to_public_key());
    }

    pub fn balance(client:&Client,pk:&bitcoin::PublicKey){
        let script=Script::new_v0_p2wpkh(&pk.wpubkey_hash().expect(&pk.pubkey_hash().to_string().add(" is missing a witness")));
        let get_balance=client.script_get_balance(&script).unwrap();
        let addr=Address::from_script(&script, NETWORK).unwrap().to_qr_uri().to_lowercase();
        println!("address: {}",addr);
        println!("confirmed: {}",get_balance.confirmed);
        println!("unconfirmed: {}",get_balance.unconfirmed)
    }

    pub fn derivation_path(change: Option<u32>,recieve:u32)->DerivationPath{
		// bip84 For the purpose-path level it uses 84'. The rest of the levels are used as defined in BIP44 or BIP49.
		// m / purpose' / coin_type' / account' / change / address_index
		let keychain=KeychainKind::External;
		return DerivationPath::from(vec![
			ChildNumber::from_hardened_idx(84).unwrap(),// purpose 
			ChildNumber::from_hardened_idx(recieve).unwrap(),// first recieve 
			ChildNumber::from_hardened_idx(0).unwrap(),// second recieve 
			ChildNumber::from_normal_idx(keychain as u32).unwrap()
		]).extend(change.map(|c|vec![ChildNumber::from_normal_idx(c).unwrap()]).unwrap_or_else(||Vec::new()));
	}
    pub fn submit_transaction(&self,change:u32,recieve:u32,send_to:String,amount:u64){
        let map_ext_keys=|c:u32|BtcK::get_ext_keys(&self.seed,&Some(BtcK::derivation_path(Some(c), recieve)),&self.secp);
        let map_addr=|ext_public_key:ExtendedPubKey|Address::p2wpkh(&ext_public_key.public_key.to_public_key(), Network::Testnet).unwrap();
        let tip:u64=100;
        let (owner_prv_k,ower_pub_k)=map_ext_keys(change);
        let change_pub_k=map_ext_keys(change).1;
        
        let history=Arc::new(self.client.script_get_history(&map_addr(ower_pub_k).script_pubkey()).expect("address history call failed"));

		let tx_in=history.iter().enumerate().map(|tx_id|
			TxIn{
				previous_output:OutPoint::new(tx_id.1.tx_hash, tx_id.0 as u32+1),
				script_sig: Script::new(),// The scriptSig must be exactly empty or the validation fails (native witness program)
				sequence: 0xFFFFFFFF,
				witness: Witness::default() 
			}
		).collect::<Vec<TxIn>>();
		
		let previous_tx=Arc::new(tx_in.iter()
        .map(|tx_id|self.client.transaction_get(&tx_id.previous_output.txid).unwrap())
        .collect::<Vec<Transaction>>());

        let input_vec=previous_tx.iter().map(|prev|{
        let mut input_tx=Input::default();
        input_tx.non_witness_utxo=Some(prev.clone());
        input_tx.witness_utxo=Some(prev.output.iter().filter(|w|
            Script::is_v0_p2wpkh(&w.script_pubkey) &&
            w.script_pubkey.eq(&Script::new_v0_p2wpkh(&ower_pub_k.public_key.to_public_key().wpubkey_hash().unwrap()))
            ).next().unwrap().clone());
            return input_tx;
        }).collect::<Vec<Input>>();

        let value=input_vec.iter().map(|tx|tx.witness_utxo.as_ref().unwrap().value).next().unwrap();

        let tx_out=vec![
                TxOut{ value: amount, script_pubkey:Address::from_str(&send_to).unwrap().script_pubkey()},
                TxOut{ value: (value-tip), script_pubkey:map_addr(change_pub_k).script_pubkey() }
            ];

        let mut transaction =Transaction{
                version: 1,
                lock_time: 0,
                input: tx_in,
                output: tx_out,
            };


        let mut xpub =BTreeMap::new();

        xpub.insert(ower_pub_k,(ower_pub_k.clone().fingerprint() ,BtcK::derivation_path(Some(change), recieve)));

        let mut output=Output::default();
        output.bip32_derivation =BTreeMap::new();
        output.bip32_derivation.insert(change_pub_k.public_key,(change_pub_k.fingerprint() ,BtcK::derivation_path(Some(change+1), recieve)));


        let mut psbt=PartiallySignedTransaction{
            unsigned_tx: transaction,
            version: 0,
            xpub,
            proprietary: BTreeMap::new(),
            unknown: BTreeMap::new(),
            outputs: vec![output],
            inputs: input_vec,
        };

        psbt.inputs[0].partial_sigs=psbt.inputs.iter().filter(|w|w.witness_utxo.is_some()).enumerate().map(|(i,input)|{

            let sighash=SighashCache::new(&mut psbt.unsigned_tx)
                .segwit_signature_hash(i, &BtcK::p2wpkh_script_code(&input.witness_utxo.as_ref().unwrap().script_pubkey), value, bitcoin::EcdsaSighashType::All).unwrap();

            let ecdsa= EcdsaSig::sighash_all(self.secp.sign_ecdsa(&Message::from_slice(&sighash).unwrap(),&owner_prv_k.private_key));

            return (ower_pub_k.to_pub().to_public_key(),ecdsa);
        }).collect::<BTreeMap<bitcoin::PublicKey,EcdsaSig>>(); 

        psbt.finalize(&self.secp).unwrap();
    }

    pub fn p2wpkh_script_code(script: &Script) -> Script {
    Builder::new()
        .push_opcode(opcodes::all::OP_DUP)
        .push_opcode(opcodes::all::OP_HASH160)
        .push_slice(&script[2..])
        .push_opcode(opcodes::all::OP_EQUALVERIFY)
        .push_opcode(opcodes::all::OP_CHECKSIG)
        .into_script()
}
}