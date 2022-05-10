use std::{str::FromStr, ops::Add, sync::Arc, collections::BTreeMap};

use bdk::{ KeychainKind, template::P2Pkh};
use bitcoin::{util::{bip32::{ExtendedPrivKey, ExtendedPubKey, DerivationPath, ChildNumber, KeySource}, sighash::SighashCache}, Address, psbt::{Input, Output, PartiallySignedTransaction}, secp256k1::{Secp256k1, All, constants, SecretKey, rand::rngs::OsRng, PublicKey, Message}, Network, Script, OutPoint, Witness, TxIn, Transaction, TxOut, Sighash, EcdsaSig, EcdsaSighashType};
use electrum_client::{Client, ElectrumApi};
use miniscript::{psbt::PsbtExt, ToPublicKey};

pub type WalletKeys=(ExtendedPubKey,KeySource);
pub type RecieveChange=(u32,u32);
pub mod p2wpk;
pub mod p2tr;
pub trait AddressSchema{
    fn map_ext_keys(&self,recieve:&ExtendedPubKey) -> Address;
    fn create_inputs(&self,wallet_keys:&ExtendedPubKey,previous_tx:&Transaction ) -> Input;
    fn wallet_purpose(&self)-> u32;
    fn new(seed: Option<String>)->Self;
    fn to_wallet(&self)->ClientWallet;
    fn create_sighash(&self,cache:&mut Transaction,s:usize,input:&Input,dp:&DerivationPath)->EcdsaSig;
}
#[derive(Clone)]
pub struct ClientWallet{ secp:Secp256k1<All>, seed:Seed }

pub struct ClientWithSchema<T:AddressSchema>{
   schema:T,
   client:Arc<Client>,
}
type Seed=[u8;constants::SECRET_KEY_SIZE];
pub const NETWORK: bitcoin::Network = Network::Testnet;
    
 impl <T: AddressSchema> ClientWithSchema<T>{
    pub fn new(schema: T)->ClientWithSchema<T> {

        return ClientWithSchema {
        schema,
         client:Arc::new(Client::new("ssl://electrum.blockstream.info:60002").expect("client connection failed !!!"))
        };
    }

    pub fn submitTx(&self,to_addr:String,amount:u64, recieve:u32,change:u32){
        let tip:u64=200;
        let secp=&self.schema.to_wallet().secp;
        let wallet=self.schema.to_wallet();
        ;
        let (signer_pub_k,(signer_finger_p, signer_dp))=wallet.create_wallet(self.schema.wallet_purpose(), recieve, change);
        let (change_pub_k,(change_finger_p, change_dp))=wallet.create_wallet(self.schema.wallet_purpose(), recieve, change+1);
        let signer_addr=self.schema.map_ext_keys(&signer_pub_k);
        let change_addr=self.schema.map_ext_keys(&change_pub_k);
        
        let history=Arc::new(self.client.script_get_history(&signer_addr.script_pubkey()).expect("address history call failed"));

		let tx_in=history.iter().enumerate().map(|(index,h)|
			TxIn{
				previous_output:OutPoint::new(h.tx_hash, index as u32+1),
				script_sig: Script::new(),// The scriptSig must be exactly empty or the validation fails (native witness program)
				sequence: 0xFFFFFFFF,
				witness: Witness::default() 
			}
		).collect::<Vec<TxIn>>();
		
		let previous_tx=Arc::new(tx_in.iter()
        .map(|tx_id|self.client.transaction_get(&tx_id.previous_output.txid).unwrap())
        .collect::<Vec<Transaction>>());
        // the above loads in the previous transactions

        // loads in our uxto that we control
        let mut input_vec=previous_tx.iter().map(|previous_tx|{
            return self.schema.create_inputs(&signer_pub_k,previous_tx );
        }).collect::<Vec<Input>>();

        // value of our utxo that we have control over since we only control one I will just get the next one 
        let value=input_vec.iter().map(|tx|tx.witness_utxo.as_ref().unwrap().value).next().unwrap();

        let change_amt=value-(amount+tip);
        // creates a transaction for the recipent
        // and another transaction for the change the rest of the uxto will be sent as a tip  
        let tx_out=vec![
                TxOut{ value: amount, script_pubkey:Address::from_str(&to_addr).unwrap().script_pubkey()},
                TxOut{ value: change_amt, script_pubkey:change_addr.script_pubkey() }
            ];


        let mut transaction =Transaction{
                version: 1,
                lock_time: 0,
                input: tx_in,
                output: tx_out,
            };
            let mut xpub =BTreeMap::new();

        // the current public key and derivation path   
        xpub.insert(signer_pub_k,(signer_finger_p ,signer_dp.clone()));

        let mut output=Output::default();
        output.bip32_derivation =BTreeMap::new();
        output.bip32_derivation.insert(change_pub_k.public_key,(change_finger_p ,change_dp));

        // we have all the infomation for the partially signed transaction 
        let mut psbt=PartiallySignedTransaction{
            unsigned_tx: transaction.clone(),
            version: 0,
            xpub,
            proprietary: BTreeMap::new(),
            unknown: BTreeMap::new(),
            outputs: vec![output],
            inputs: input_vec.clone(),
        };
    // signs our the spending inputs to unlock from the previous script
        psbt.inputs[0].partial_sigs=input_vec.clone().iter().filter(|w|w.witness_utxo.is_some()).enumerate().map(|(i,input)|{
             
            return (signer_pub_k.to_pub().to_public_key(),self.schema.create_sighash(&mut transaction, i, input, &signer_dp));
        }).collect::<BTreeMap<bitcoin::PublicKey,EcdsaSig>>(); 

        //  finailize the transaction
        let complete=psbt.clone().finalize(&secp).unwrap();
        // self.client.transaction_broadcast(&complete.clone().extract_tx()).unwrap();
        dbg!(complete.clone());
    }

    pub fn print_balance(&self,recieve:u32, change:u32){
        let (ext_pub,_)=self.schema.to_wallet().create_wallet(self.schema.wallet_purpose(), recieve, change);
        let address=self.schema.map_ext_keys(&ext_pub);
        let get_balance=self.client.script_get_balance(&address.script_pubkey()).unwrap();
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
			ChildNumber::from_normal_idx(keychain as u32).unwrap()
		]);

        let ext_prv=ExtendedPrivKey::new_master(NETWORK, &self.seed).unwrap().derive_priv(&self.secp,&path).unwrap();

        let child=[ChildNumber::from_normal_idx(index).unwrap()];
        
        let ext_pub=ExtendedPubKey::from_priv(&self.secp, &ext_prv)
        .derive_pub(&self.secp, &child).unwrap();
         return ( ext_pub,(ext_pub.fingerprint(),path.extend(child)));
    }
}
//1816211 