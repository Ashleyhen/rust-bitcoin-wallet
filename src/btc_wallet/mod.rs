use std::{str::FromStr, ops::Add, sync::Arc, collections::BTreeMap};

use bdk::{KeychainKind, bitcoin::blockdata::transaction};
use bitcoin::{util::{bip32::{ExtendedPrivKey, ExtendedPubKey, DerivationPath, ChildNumber, KeySource}, sighash::SighashCache, bip143::SigHashCache, taproot::{TapLeafHash, LeafVersion}}, Address, psbt::{Input, Output, PartiallySignedTransaction}, secp256k1::{Secp256k1, All, constants, SecretKey, rand::rngs::OsRng, PublicKey, Message}, Network, Script, OutPoint, Witness, TxIn, Transaction, TxOut, Sighash, EcdsaSig, EcdsaSighashType};
use electrum_client::{Client, ElectrumApi};
use miniscript::{psbt::PsbtExt, ToPublicKey};

use self::storage::UxtoStorage;



pub type WalletKeys=(ExtendedPubKey,KeySource);
pub type RecieveChange=(u32,u32);
pub mod p2wpkh;
pub mod p2tr;
mod storage;

pub trait AddressSchema{
    fn map_ext_keys(&self,recieve:&ExtendedPubKey) -> Address;
    fn wallet_purpose(&self)-> u32;
    fn new(seed: Option<String>)->Self;
    fn to_wallet(&self)->ClientWallet;
}

pub trait UnlockPreviousUTXO{
    fn prv_tx_input(&self,wallet_keys:&ExtendedPubKey,previous_tx:&Transaction ) ->Input;
    // fn prv_psbt_input(&self,prev_transaction:&mut Transaction,index_input:usize,input:&Input,dp:&DerivationPath)->EcdsaSig;
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
        let (signer_pub_k,(signer_finger_p, signer_dp))=wallet.create_wallet(self.schema.wallet_purpose(), recieve, change);
        let (change_pub_k,(change_finger_p, change_dp))=wallet.create_wallet(self.schema.wallet_purpose(), recieve, change+1);
        let signer_addr=self.schema.map_ext_keys(&signer_pub_k);
        let change_addr=self.schema.map_ext_keys(&change_pub_k);
        // dbg!(signer_addr.script_pubkey().clone());
        let history=Arc::new(self.rpc_call.script_get_history(&signer_addr.script_pubkey()).expect("address history call failed"));

		let tx_in=history.iter().enumerate().map(|(index,h)|
			TxIn{
				previous_output:OutPoint::new(h.tx_hash, index as u32+1),
				script_sig: Script::new(),// The scriptSig must be exactly empty or the validation fails (native witness program)
				sequence: 0xFFFFFFFF,
				witness: Witness::default() 
			}
		).collect::<Vec<TxIn>>();
		
		let previous_tx=Arc::new(tx_in.iter()
        .map(|tx_id|self.rpc_call.transaction_get(&tx_id.previous_output.txid).unwrap())
        .collect::<Vec<Transaction>>());
        // the above loads in the previous transactions

        // loads in our uxto that we control
    let  (utxo_func, input_vec):(Vec<Arc<dyn UnlockPreviousUTXO>>,Vec<Input>)=previous_tx.iter().map(|previous_tx|{
            return  previous_tx.output.iter()
             .filter(|tx_in|{ 
                return tx_in.script_pubkey.eq(&self.schema.map_ext_keys(&signer_pub_k).script_pubkey());
             }).map(|tx_out|{
            let prv_utxo_func =wallet.clone().unlock_tx_functions(tx_out.script_pubkey.clone() );
                return (prv_utxo_func.clone(),prv_utxo_func.prv_tx_input(&signer_pub_k,previous_tx ));
            }).unzip();
        }).reduce(|previous,current|
            return ([previous.0,current.0].concat(),[previous.1,current.1].concat())
        ).unwrap();
        
    // value of our utxo that we have control over since we only control one I will just get the next one 
        let value=input_vec.iter().map(|tx|tx.witness_utxo.as_ref().unwrap().value).next().unwrap();
        // let value=1000;

        let change_amt=value-(amount+tip);
        // creates a transaction for the recipent
        // and another transaction for the change the rest of the uxto will be sent as a tip  
        let tx_out=vec![
            if change_amt>=tip { 
                Some(TxOut{ value: change_amt, script_pubkey:change_addr.script_pubkey() })
            } else {None},
            Some(TxOut{ value: amount, script_pubkey:Address::from_str(&to_addr).unwrap().script_pubkey()})
            ].iter()
        .filter(|f|f.is_some())
        .map(|f|f.clone().unwrap()).collect();


        let mut transaction =Transaction{
                version: 0,
                lock_time: 0,
                input: tx_in,
                output: tx_out
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
            version: 1,
            xpub,
            proprietary: BTreeMap::new(),
            unknown: BTreeMap::new(),
            outputs: vec![output],
            inputs: utxo_func.into_iter().enumerate().map(|(i,func)|
                func.prv_psbt_input(&mut transaction, &input_vec[i], i,&(signer_pub_k,(signer_finger_p, signer_dp.clone())))).collect::<Vec<Input>>(),
        };
        ;
// psbt.sighash_msg(0, &mut SighashCache::new(&mut transaction), 
// Some(TapLeafHash::from_script(&input_vec[0].witness_utxo.as_ref().unwrap().script_pubkey, LeafVersion::TapScript))).unwrap();
    // signs our the spending inputs to unlock from the previous script
    
        // psbt.inputs[0].partial_sigs=input_vec.clone().iter().filter(|w|w.witness_utxo.is_some()).enumerate().map(|(i,input)|{
        //     return (signer_pub_k.to_pub().to_public_key(),utxo_func[i].clone().prv_psbt_input(&mut transaction, i, input, &signer_dp));
        // }).collect::<BTreeMap<bitcoin::PublicKey,EcdsaSig>>(); 

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

pub fn unlock_tx_functions(self, script:Script)->Arc< dyn UnlockPreviousUTXO>{
    if script.is_v0_p2wpkh() {
        return  Arc::new(p2wpkh::P2PWKh(self));
    }else if script.is_v1_p2tr(){
        return Arc::new(p2tr::P2TR(self));
    }
    panic!("script type unknown");
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

