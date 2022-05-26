use std::{collections::BTreeMap};

use bitcoin::{Address, util::{bip32::{DerivationPath, ExtendedPubKey, ExtendedPrivKey, KeySource}, sighash::{SighashCache, Prevouts}, taproot::{TapLeafHash,LeafVersion::TapScript}}, Transaction, psbt::Input, Script, SchnorrSighashType, SchnorrSig, secp256k1::{schnorr::Signature, Message, schnorrsig::PublicKey }, KeyPair, TxOut, blockdata::{script::Builder, opcodes},  XOnlyPublicKey, schnorr::{UntweakedPublicKey, TweakedPublicKey, TapTweak}, TxIn};
use miniscript::{interpreter::KeySigPair, ToPublicKey};

use super::{AddressSchema, ClientWallet, NETWORK, WalletKeys, utils::{UnlockAndSend, TxOutMap}};

#[derive( Clone)]
pub struct P2TR(pub ClientWallet);

impl AddressSchema for P2TR{
    fn map_ext_keys(&self,recieve:&bitcoin::util::bip32::ExtendedPubKey) -> bitcoin::Address { ;
		return Address::p2tr(&self.0.secp, recieve.to_x_only_pub(), None, NETWORK);
    }
    
    fn new(seed: Option<String>, recieve:u32, change:u32)->Self {
		return P2TR(ClientWallet::new(seed,recieve,change));
    }

    fn to_wallet(&self)->ClientWallet {
		return self.0.clone();
    }

    fn wallet_purpose(&self)-> u32 {
        return 341;
    }


    fn prv_tx_input(&self,previous_tx:Vec<Transaction>,current_tx:Transaction) ->(Vec<Input>, Transaction) {
        let wallet_key=self.0.create_wallet(self.wallet_purpose(),self.0.recieve,self.0.change);

        let (signer_pub_k,(signer_finger_p, signer_dp))=wallet_key.clone();

        let secp=&self.0.secp;
        let ext_prv=ExtendedPrivKey::new_master(NETWORK, &self.0.seed).unwrap().derive_priv(&secp, &signer_dp).unwrap();

        let input_list:Vec<Input>=previous_tx.iter().enumerate().map(|(i, previous_tx)|{
        let tweaked_key_pair=ext_prv.to_keypair(&secp).tap_tweak(&secp,None).into_inner();       
       
        let new_input=self.process_tx(i, previous_tx, current_tx.clone(),tweaked_key_pair);

          

        return new_input;
        }).collect();
        return (input_list,current_tx);
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
impl P2TR{
  pub fn process_tx(&self,i: usize,previous_tx:&Transaction,current_tx:Transaction,key_pair:KeyPair)->(Input){
let wallet_key=self.0.create_wallet(self.wallet_purpose(),self.0.recieve,self.0.change);

        let (signer_pub_k,(signer_finger_p, signer_dp))=wallet_key.clone();

        let secp=&self.0.secp;

          let tx_output:Vec<TxOut>= previous_tx.output.clone().iter()
          .filter(|tx_out|UnlockAndSend::new(self,wallet_key.clone()).find_relevent_utxo(tx_out)).map(|tx_out|tx_out.clone()).collect();

          let tap_leaf_hash_list=tx_output.clone().iter().map(|f|TapLeafHash::from_script(&f.script_pubkey, TapScript)).collect::<Vec<TapLeafHash>>();

          let uxto=&tx_output.clone()[0];

          let mut tap_key_origin=BTreeMap::new();

            tap_key_origin.insert
            (signer_pub_k.to_x_only_pub(),
            (tap_leaf_hash_list,(signer_finger_p.clone(),signer_dp.clone())));

          let sig_hash=SighashCache::new(&mut current_tx.clone())
                  .taproot_key_spend_signature_hash( i, &Prevouts::All(&tx_output), SchnorrSighashType::AllPlusAnyoneCanPay).unwrap();
                  let msg=Message::from_slice(&sig_hash).unwrap();

          let signed_shnorr=secp.sign_schnorr(&msg, &key_pair);

          let schnorr_sig=SchnorrSig{sig:signed_shnorr, hash_ty:SchnorrSighashType::AllPlusAnyoneCanPay};
          let mut new_input=Input::default() ;
          new_input.witness_utxo=Some(uxto.clone());
          new_input.tap_key_origins=tap_key_origin;
          new_input.tap_internal_key=Some(signer_pub_k.to_x_only_pub());
          new_input.tap_key_sig=Some(schnorr_sig.clone());
          new_input.non_witness_utxo=Some(previous_tx.clone());
          secp.verify_schnorr(&signed_shnorr, &msg, &XOnlyPublicKey::from_slice(&uxto.script_pubkey[2..]).unwrap()).is_ok();
          return (new_input);
  
  }
}