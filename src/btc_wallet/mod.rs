use std::{str::FromStr, ops::Add, sync::Arc, collections::BTreeMap};

use bdk::KeychainKind;
use bitcoin::{util::{bip32::{ExtendedPrivKey, ExtendedPubKey, DerivationPath, ChildNumber, KeySource}, sighash::SighashCache, bip143::SigHashCache, taproot::{TapLeafHash, LeafVersion}}, Address, psbt::{Input, Output, PartiallySignedTransaction}, secp256k1::{Secp256k1, All, constants, SecretKey, rand::rngs::OsRng, PublicKey, Message}, Network, Script, OutPoint, Witness, TxIn, Transaction, TxOut, Sighash, EcdsaSig, EcdsaSighashType};
use electrum_client::{Client, ElectrumApi};
use miniscript::{psbt::PsbtExt, ToPublicKey};

use crate::btc_wallet::utils::UnlockAndSend;
mod utils;

pub type WalletKeys=(ExtendedPubKey,KeySource);
pub mod p2wpkh;
pub mod p2tr;

pub trait AddressSchema{
    fn map_ext_keys(&self,recieve:&ExtendedPubKey) -> Address;
    fn wallet_purpose(&self)-> u32;
    fn new(seed: Option<String>)->Self;
    fn to_wallet(&self)->ClientWallet;
    fn prv_tx_input(&self,amount:u64,to_addr:String,change:Script,wallet_keys:&WalletKeys,previous_tx:Vec<Transaction>,current_input:Vec<TxIn> ) ->(Vec<Input>, Transaction);
    fn prv_psbt_input(&self,prev_transaction:&mut Transaction,input:&Input,i:usize,wallet_keys:&WalletKeys)->Input;
}




#[derive(Clone)]
pub struct ClientWallet{ secp:Secp256k1<All>, seed:Seed }

pub struct ClientWithSchema<T:AddressSchema>{
   schema:T,
   rpc_call:Arc<Client>,
}
type Seed=[u8;constants::SECRET_KEY_SIZE];
pub const NETWORK: bitcoin::Network = Network::Testnet;

 impl <T: AddressSchema> ClientWithSchema<T>{
    pub fn new(schema: T)->ClientWithSchema<T> {

        return ClientWithSchema {
        schema,
         rpc_call:Arc::new(Client::new("ssl://electrum.blockstream.info:60002").expect("client connection failed !!!"))
        };
    }

    pub fn submit_tx(&self,to_addr:String,amount:u64, recieve:u32,change:u32){
        let tip:u64=200;
        let secp=&self.schema.to_wallet().secp;
        let wallet=self.schema.to_wallet();
        let signer=wallet.create_wallet(self.schema.wallet_purpose(), recieve, change);
        let (signer_pub_k,(signer_finger_p,signer_dp))=signer.clone();
        let (change_pub_k,(change_finger_p, change_dp))=wallet.create_wallet(self.schema.wallet_purpose(), recieve, change+1);
        let signer_addr=self.schema.map_ext_keys(&signer_pub_k);
        let change_addr=self.schema.map_ext_keys(&change_pub_k);

        let history=Arc::new(self.rpc_call.script_get_history(&signer_addr.script_pubkey()).expect("address history call failed"));

		let tx_in=history.iter().enumerate().map(|(index,h)|{
			return TxIn{
				previous_output:OutPoint::new(h.tx_hash, index as u32+1),
				script_sig: Script::new(),// The scriptSig must be exactly empty or the validation fails (native witness program)
				sequence: 0xFFFFFFFF,
				witness: Witness::default() 
			}

        }
		).collect::<Vec<TxIn>>();
		
		let previous_tx=Arc::new(tx_in.iter()
        .map(|tx_id|self.rpc_call.transaction_get(&tx_id.previous_output.txid).unwrap())
        .collect::<Vec<Transaction>>());
      
            let unlock_and_send=UnlockAndSend::new(self.schema, signer.clone());
        let current_tx= |a:&dyn FnMut(&bitcoin::TxOut)|unlock_and_send.initialize(amount, to_addr, change_addr.script_pubkey(), tx_in, previous_tx.to_vec(),a);
let (input_vec, current_tx)=self.schema.prv_tx_input(amount,to_addr,change_addr.script_pubkey(),&signer,previous_tx.to_vec(),tx_in );
       
            let mut xpub =BTreeMap::new();

        // the current public key and derivation path   
        xpub.insert(signer_pub_k,(signer_finger_p ,signer_dp.clone()));

        let mut output=Output::default();
        output.bip32_derivation =BTreeMap::new();
        output.bip32_derivation.insert(change_pub_k.public_key,(change_finger_p ,change_dp));

        // we have all the infomation for the partially signed transaction 
        let mut psbt=PartiallySignedTransaction{
            unsigned_tx: current_tx.clone(),
            version: 1,
            xpub,
            proprietary: BTreeMap::new(),
            unknown: BTreeMap::new(),
            outputs: vec![output],
            inputs:input_vec
        };

        // dbg!(psbt.clone());
        //  finailize the transaction
        let complete=psbt.clone().finalize(&secp).unwrap();
        // self.rpc_call.transaction_broadcast(&complete.clone().extract_tx()).unwrap();
        dbg!(complete.clone());
    }

    pub fn print_balance(&self,recieve:u32, change:u32){
        let (ext_pub,_)=self.schema.to_wallet().create_wallet(self.schema.wallet_purpose(), recieve, change);
        let address=self.schema.map_ext_keys(&ext_pub);
        let get_balance=self.rpc_call.script_get_balance(&address.script_pubkey()).unwrap();
        println!("address: {}",address);
        println!("confirmed: {}",get_balance.confirmed);
        println!("unconfirmed: {}",get_balance.unconfirmed)
    }

}
impl ClientWallet{

    pub fn new(secret_seed:Option<String>)-> ClientWallet{
            return ClientWallet{
                secp:Secp256k1::new(),
                seed: (match secret_seed{
                Some(secret)=> SecretKey::from_str(&secret).expect("failed to initalize seed, check if the seed is valid"),
                _ => SecretKey::new(&mut OsRng::new().expect("failed to create a random number generator !!! ") ) }).secret_bytes(),
            
            }
        }



 fn create_wallet(&self,purpose:u32, recieve:u32,index:u32) -> WalletKeys {
		// bip84 For the purpose-path level it uses 84'. The rest of the levels are used as defined in BIP44 or BIP49.
		// m / purpose' / coin_type' / account' / change / address_index
		let keychain=KeychainKind::External;
        let path=DerivationPath::from( vec![
			ChildNumber::from_hardened_idx(purpose).unwrap(),// purpose 
			ChildNumber::from_hardened_idx(recieve).unwrap(),// first recieve 
			ChildNumber::from_hardened_idx(0).unwrap(),// second recieve 
			ChildNumber::from_normal_idx(keychain as u32).unwrap(),
            ChildNumber::from_normal_idx(index).unwrap()
		]);
        let ext_prv=ExtendedPrivKey::new_master(NETWORK, &self.seed).unwrap().derive_priv(&self.secp,&path).unwrap();
        let ext_pub=ExtendedPubKey::from_priv(&self.secp, &ext_prv);

         return ( ext_pub,(ext_pub.fingerprint(),path));
    }
    
}
struct WalletPath{
    purpose:u32,
    recieve:u32,
    second_recieve:u32,
    key_chain:u32,
    change:u32
}

// fn path<F>(change:F) 
//         where F: FnOnce(Script) ->dyn UnlockPreviousUTXO{
        
// }

