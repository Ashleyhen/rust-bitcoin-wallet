use std::{str::FromStr, ops::Add, sync::Arc, collections::BTreeMap};

use bdk::{ KeychainKind, template::P2Pkh};
use bitcoin::{util::{bip32::{ExtendedPrivKey, ExtendedPubKey, DerivationPath, ChildNumber, KeySource}, sighash::SighashCache}, Address, psbt::{Input, Output, PartiallySignedTransaction}, secp256k1::{Secp256k1, All, constants, SecretKey, rand::rngs::OsRng, PublicKey}, Network, Script, OutPoint, Witness, TxIn, Transaction, TxOut, Sighash, EcdsaSig};
use electrum_client::{Client, ElectrumApi};
use miniscript::{psbt::PsbtExt, ToPublicKey};

pub type WalletKeys=(ExtendedPrivKey,ExtendedPubKey,KeySource);
pub type RecieveChange=(u32,u32);
pub mod p2wpk;
pub trait AddressSchema{
    fn map_ext_keys(&self,recieve:&ExtendedPubKey) -> Address;
    fn create_inputs(&self,wallet_keys:&ExtendedPubKey,previous_tx:&Transaction ) -> Input;
    fn new_wallet(&self,recieve:u32,change:u32)-> WalletKeys;
    fn new(seed: Option<String>)->Self;
    fn to_wallet(&self)->ClientWallet;
    fn create_sighash(&self,cache:&mut SighashCache<&mut Transaction>,s:usize,input:&Input)->EcdsaSig;
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
        let tip=100;

       let (_, ext_pub,(fingerprint,derived_path))= self.schema.new_wallet(recieve, change);

       let history=Arc::new(self.client.script_get_history(&self.schema.map_ext_keys(&ext_pub).script_pubkey()).unwrap());

       let tx_in=history.iter().enumerate().map(|(i, h)|{
            TxIn{
                previous_output:OutPoint::new(h.tx_hash,i as u32+1),
                script_sig:Script::new(),
				sequence: 0xFFFFFFFF,
                witness: Witness::default()
            }
        }).collect::<Vec<TxIn>>();

        let previous_tx=Arc::new(tx_in.iter()
        .map(|tx_id|self.client.transaction_get(&tx_id.previous_output.txid).unwrap())
        .collect::<Vec<Transaction>>());

        let inputs=previous_tx.iter()
        .map(|prev|self.schema.create_inputs(&ext_pub,prev)).collect::<Vec<Input>>();

// if amount are same use script above
    let (_,chg_ext_pub,(chg_fp,chg_dp) )=self.schema.new_wallet(recieve,change+1);

    let recepient=previous_tx.clone().iter().flat_map(|tx|{
    return tx.output.iter()
            .filter(|tx_out| tx_out.script_pubkey.eq(&self.schema.map_ext_keys(&ext_pub).script_pubkey()))
            .flat_map(|tx_out|
                [ TxOut{value:tx_out.value-(amount+tip),script_pubkey:self.schema.map_ext_keys(&chg_ext_pub).script_pubkey()},
                TxOut{value:amount, script_pubkey: Address::from_str(&to_addr).unwrap().script_pubkey() } ]
            )
            .collect::<Vec<TxOut>>();
        }).collect::<Vec<TxOut>>();

        let mut transaction= Transaction{ 
            version: 1, 
            lock_time:0, 
            output:recepient,
            input: tx_in, };
        
        let mut xpub =BTreeMap::new();
        xpub.insert(ext_pub,(fingerprint,derived_path.clone()));

        
         let mut output=Output::default();
        
        output.bip32_derivation=BTreeMap::new();
        output.bip32_derivation.insert(ext_pub.public_key, (fingerprint,chg_dp.clone()));

        let mut psbt=PartiallySignedTransaction{
            unsigned_tx:transaction.clone(),
            version:0,
            xpub,
            proprietary:BTreeMap::new(),
            unknown:BTreeMap::new(),
            outputs:vec![output],
            inputs
        };

       psbt.inputs[0].partial_sigs= psbt.inputs.clone().iter().filter(|w|w.witness_utxo.is_some()).enumerate().map(|(i,inpts)|{

        return (ext_pub.to_pub(),self.schema.create_sighash(&mut SighashCache::new(&mut transaction), i, &inpts));
        }).collect::<BTreeMap<bitcoin::PublicKey,EcdsaSig>>();
        dbg!(psbt.clone());
        psbt.finalize(&self.schema.to_wallet().secp).unwrap();
    }

    pub fn print_balance(&self,recieve:u32, change:u32){
        let (_,ext_pub,_)=self.schema.new_wallet(recieve, change);
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


    fn get_ext_keys(&self,dp:&DerivationPath)->(ExtendedPrivKey){
        return ExtendedPrivKey::new_master(NETWORK, &self.seed).unwrap().derive_priv(&self.secp,&dp).unwrap(); 
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
         return (ext_prv, ext_pub,(ext_pub.fingerprint(),path.extend(child)));
    }
}
//1816211 