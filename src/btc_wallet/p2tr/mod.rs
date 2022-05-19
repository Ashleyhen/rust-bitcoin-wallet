use std::{collections::BTreeMap, borrow::Borrow};

use bitcoin::{Address, util::{bip32::{DerivationPath, ExtendedPubKey, ExtendedPrivKey, KeySource}, sighash::{SighashCache, Prevouts}, taproot::{TapLeafHash,LeafVersion::TapScript}}, Transaction, psbt::Input, Script, SchnorrSighashType, SchnorrSig, secp256k1::{schnorr::Signature, Message, schnorrsig::PublicKey }, KeyPair, TxOut, blockdata::{script::Builder, opcodes},  XOnlyPublicKey, schnorr::{UntweakedPublicKey, TweakedPublicKey}};
use miniscript::{interpreter::KeySigPair, ToPublicKey};

use super::{AddressSchema, ClientWallet, NETWORK, WalletKeys, utils::UnlockAndSend};

#[derive( Clone)]
pub struct P2TR(pub ClientWallet);

impl AddressSchema for P2TR{
    fn map_ext_keys(&self,recieve:&bitcoin::util::bip32::ExtendedPubKey) -> bitcoin::Address { ;
		return Address::p2tr(&self.0.secp, recieve.to_x_only_pub(), None, NETWORK);
    }
    
    fn new(seed: Option<String>)->Self {
		return P2TR(ClientWallet::new(seed));
    }

    fn to_wallet(&self)->ClientWallet {
		return self.0.clone();
    }

    fn wallet_purpose(&self)-> u32 {
        return 341;
    }
    /*
  fn prv_tx_input(&self,ext_pub:&ExtendedPubKey,previous_tx:&bitcoin::Transaction ) ->bitcoin::psbt::Input {

        let mut input_tx= Input::default();
        let tr_script=Script::new_v1_p2tr(&self.0.secp,ext_pub.to_x_only_pub(),None);
        input_tx.non_witness_utxo=Some((*previous_tx).clone());
        
        input_tx.witness_utxo=Some(previous_tx.output.iter()
        .filter(|w|w.script_pubkey.eq(&tr_script)).next().unwrap().clone());
        return input_tx;
      }
*/
      fn prv_psbt_input(&self,prev_transaction:&mut Transaction,input_outpoints: &Input,i:usize,wallet_keys:&WalletKeys)->Input {
        let (signer_pub_k,(signer_finger_p, signer_dp))=wallet_keys;
        let secp=&self.0.secp;
        let ext_prv=ExtendedPrivKey::new_master(NETWORK, &self.0.seed).unwrap().derive_priv(&secp, &signer_dp).unwrap();

        let out_tx_list=prev_transaction.output.iter().filter(| out|
          self.map_ext_keys(signer_pub_k)
          .eq(&Address::from_script(&out.script_pubkey,NETWORK).unwrap())).map(|out|out.clone())
          .map(|f| TxOut{
          value: f.value,
          script_pubkey:Script::new_v1_p2tr_tweaked( TweakedPublicKey::dangerous_assume_tweaked(signer_pub_k.to_x_only_pub())) })
          .collect::<Vec<TxOut>>() ;
              
        let mut tap_key_origin=BTreeMap::new();
        let script_list=out_tx_list.clone()
          .iter().map(|f|TapLeafHash::from_script(&f.script_pubkey, TapScript)).collect::<Vec<TapLeafHash>>();
          tap_key_origin.insert
          (signer_pub_k.to_x_only_pub(),
          (script_list,(signer_finger_p.clone(),signer_dp.clone())));
        
        let sighash=SighashCache::new(&mut prev_transaction.clone()).taproot_key_spend_signature_hash(
                i, &Prevouts::All(&out_tx_list), SchnorrSighashType::Default).unwrap();
                
        let msg=Message::from_slice(&sighash).unwrap();
        let signed_shnorr=secp.sign_schnorr(&msg, &ext_prv.to_keypair(secp));
        let schnorr_sig=SchnorrSig{ sig:signed_shnorr ,hash_ty: SchnorrSighashType::Default };
        let uxto=out_tx_list.iter().next().map(|f|f.clone()).unwrap();

        let mut new_input=input_outpoints.clone() ;
        new_input.witness_utxo=Some(uxto.clone());
        new_input.tap_key_origins=tap_key_origin;
        new_input.tap_internal_key=Some(signer_pub_k.to_x_only_pub());
        new_input.tap_key_sig=Some(schnorr_sig.clone());
        let is_successful=secp.verify_schnorr(&signed_shnorr, &msg, &XOnlyPublicKey::from_slice(&uxto.script_pubkey[2..]).unwrap()).is_ok();
      return new_input;

    }

    fn prv_tx_input(&self,amount:u64,to_addr:String,change_addr:Script,wallet_keys:&WalletKeys,previous_tx:Vec<Transaction>) ->(Vec<Input>, Transaction) {

        let (signer_pub_k,(signer_finger_p, signer_dp))=wallet_keys;
        let secp=&self.0.secp;
        let ext_prv=ExtendedPrivKey::new_master(NETWORK, &self.0.seed).unwrap().derive_priv(&secp, signer_dp).unwrap();
        let unlock_and_send=UnlockAndSend::new(self.clone(), (*wallet_keys).clone());
        let  tx_out_mapping =|f:&TxOut| TxOut{
          value: f.value,
          script_pubkey:Script::new_v1_p2tr_tweaked( TweakedPublicKey::dangerous_assume_tweaked(signer_pub_k.to_x_only_pub())) };

        let current_tx= unlock_and_send.initialize(amount, to_addr, change_addr, previous_tx.clone(),  tx_out_mapping);

        // confirm
        let input_list:Vec<Input>=previous_tx.iter().enumerate().map(|(i, previous_tx)|{
        let utxo_list=unlock_and_send.find_relevent_utxo(previous_tx, tx_out_mapping);
        let uxto=&utxo_list[0];
        let script_list=utxo_list.clone().iter().map(|f|TapLeafHash::from_script(&f.script_pubkey, TapScript)).collect::<Vec<TapLeafHash>>();
        let mut tap_key_origin=BTreeMap::new();
          tap_key_origin.insert
          (signer_pub_k.to_x_only_pub(),
          (script_list,(signer_finger_p.clone(),signer_dp.clone())));

                
              let sig_hash=SighashCache::new(&mut current_tx.clone())
                .taproot_key_spend_signature_hash( i, &Prevouts::All(&utxo_list), SchnorrSighashType::Default).unwrap();
                let msg=Message::from_slice(&sig_hash).unwrap();

                let signed_shnorr=secp.sign_schnorr(&msg, &ext_prv.to_keypair(secp));

                let schnorr_sig=SchnorrSig{sig:signed_shnorr, hash_ty:SchnorrSighashType::Default};
               let pub_key=signer_pub_k.public_key.to_public_key();
        let mut new_input=Input::default() ;
        new_input.witness_utxo=Some(uxto.clone());
        new_input.tap_key_origins=tap_key_origin;
        new_input.tap_internal_key=Some(signer_pub_k.to_x_only_pub());
        new_input.tap_key_sig=Some(schnorr_sig.clone());
new_input.non_witness_utxo=Some(previous_tx.clone());
                new_input.witness_utxo=Some((utxo_list[0]).clone());
        let is_successful=secp.verify_schnorr(&signed_shnorr, &msg, &XOnlyPublicKey::from_slice(&uxto.script_pubkey[2..]).unwrap()).is_ok();
 

        return new_input;
        }).collect();
        return (input_list,current_tx);
    }
  }
       
        // let mut input_tx= Input::default();
        // let tr_script=Script::new_v1_p2tr(&self.0.secp,ext_pub.to_x_only_pub(),None);
        // input_tx.non_witness_utxo=Some((*previous_tx).clone());
        
        // input_tx.witness_utxo=Some(previous_tx.output.iter()
        // .filter(|w|w.script_pubkey.eq(&tr_script)).next().unwrap().clone());
        // return input_tx;
 

  pub fn p2wpkh_script_code(script: &Script) -> Script {
    Builder::new()
        .push_opcode(opcodes::all::OP_DUP)
        .push_opcode(opcodes::all::OP_HASH160)
        .push_slice(&script[2..])
        .push_opcode(opcodes::all::OP_EQUALVERIFY)
        .push_opcode(opcodes::all::OP_CHECKSIG)
        .into_script()
    
}
