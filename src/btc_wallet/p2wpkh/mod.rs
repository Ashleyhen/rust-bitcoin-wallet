use std::{str::FromStr, collections::BTreeMap, io::Read, sync::Arc};

use bitcoin::{util::{bip32::{DerivationPath, ExtendedPubKey, ExtendedPrivKey, KeySource}, sighash::SighashCache}, psbt::{Output, Input}, Address, Transaction, TxOut, Script, blockdata::{script::Builder, opcodes}, secp256k1::{SecretKey, Message}, EcdsaSig, EcdsaSighashType, Sighash, TxIn};
use miniscript::ToPublicKey;


use super::{ClientWallet, AddressSchema,  NETWORK,  WalletKeys, utils::{UnlockAndSend, TxOutMap}, OutPutMap};

#[derive( Clone)]
pub struct P2PWKh( pub  ClientWallet ); 

impl AddressSchema for P2PWKh{

    fn map_ext_keys(&self,recieve:&ExtendedPubKey) -> bitcoin::Address {
        return Address::p2wpkh(&recieve.public_key.to_public_key(), NETWORK).unwrap();
    }

    fn new(seed: Option<String>,recieve:u32, change:u32)->Self {
        return P2PWKh(ClientWallet::new(seed,recieve,change));
    }

    fn to_wallet(&self)->ClientWallet {
        return self.0.clone();
    }

    fn wallet_purpose(&self)-> u32 {
        return 84;
    }

    fn prv_tx_input(&self,previous_tx:Vec<Transaction>,current_tx:Transaction) -> (Vec<Input>,Transaction) {

        let wallet_keys=self.0.create_wallet(self.wallet_purpose(),self.0.recieve,self.0.change);
        let (signer_pub_k,(_, signer_dp))=wallet_keys.clone();
        let secp=&self.0.secp;
        let ext_prv=ExtendedPrivKey::new_master(NETWORK, &self.0.seed).unwrap().derive_priv(&secp, &signer_dp).unwrap();
        // let unlock_and_send=UnlockAndSend::new(&self.clone(), (wallet_keys).clone());

        

        // let current_tx=Transaction{ version:0, lock_time:0, input: tx_input, output:tx_out.clone() };
        // confirm
        let input_list:Vec<Input>=previous_tx.iter().enumerate().map(|(i, previous_tx)|{
            let mut b_tree=BTreeMap::new();
            let mut input_tx=Input::default();
                input_tx.non_witness_utxo=Some(previous_tx.clone());

            previous_tx.output.iter().for_each(|witness|{
                input_tx.witness_utxo=Some(witness.clone());
                let sig_hash=SighashCache::new(&mut current_tx.clone())
                .segwit_signature_hash( i, &p2wpkh_script_code(&witness.script_pubkey), witness.value, EcdsaSighashType::All).unwrap();
                let msg=Message::from_slice(&sig_hash).unwrap();
                let sig=EcdsaSig::sighash_all(secp.sign_ecdsa(&msg,&ext_prv.private_key));
                let pub_key=signer_pub_k.public_key.to_public_key();
                b_tree.insert(pub_key,sig);
            });
        input_tx.partial_sigs=b_tree;
        return input_tx;
        }).collect();
        return (input_list,current_tx);
        }

       
        
        




    fn prv_psbt_input(&self,prev_transaction:&mut Transaction,old_input: &Input,i:usize,wallet_keys:&WalletKeys)->Input {

        let (signer_pub_k,(_, signer_dp))=wallet_keys;
        let secp=&self.0.secp;
        let ext_prv=ExtendedPrivKey::new_master(NETWORK, &self.0.seed).unwrap().derive_priv(&secp, signer_dp).unwrap();
        
        let sighash=old_input.witness_utxo.as_ref().map(|tx|{
            return SighashCache::new( prev_transaction).segwit_signature_hash(
                i, &p2wpkh_script_code(&tx.script_pubkey), tx.value, EcdsaSighashType::All)
        }).unwrap().unwrap();
 
        let mut b_tree=BTreeMap::new();
        b_tree.insert(signer_pub_k.public_key.to_public_key(),EcdsaSig::sighash_all(secp.sign_ecdsa(&Message::from_slice(&sighash).unwrap(),&ext_prv.private_key)));
        let mut new_input=old_input.clone();
        new_input.partial_sigs=b_tree;
        return new_input;
    }

     fn map_tx( &self,tx_out:&TxOut)->  TxOut {
        return tx_out.clone();
    }


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
fn path<F>(change:F) 
        where F: FnOnce() ->DerivationPath
    {
        


    }