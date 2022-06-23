use std::{collections::BTreeMap, str::FromStr, borrow::Borrow};

use bitcoin::{Address, util::{bip32::{DerivationPath, ExtendedPubKey, ExtendedPrivKey, KeySource}, sighash::{SighashCache, Prevouts}, taproot::{TapLeafHash,LeafVersion::TapScript}, address::Payload}, Transaction, psbt::Input, Script, SchnorrSighashType, SchnorrSig, secp256k1::{schnorr::Signature, Message, schnorrsig::PublicKey, Parity }, KeyPair, TxOut, blockdata::{script::Builder, opcodes},  XOnlyPublicKey, schnorr::{UntweakedPublicKey, TweakedPublicKey, TapTweak}, TxIn};
use miniscript::{interpreter::KeySigPair, ToPublicKey};

use super::{AddressSchema, ClientWallet, NETWORK, WalletKeys, utils::{UnlockAndSend}, Broadcast_op};

#[derive( Clone)]
pub struct P2TR(pub ClientWallet);

impl AddressSchema for P2TR{
    fn map_ext_keys(&self,recieve:&bitcoin::util::bip32::ExtendedPubKey) -> bitcoin::Address { 
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

    fn prv_tx_input(&self,previous_tx:Vec<Transaction>,current_tx:Transaction,broadcast_op:&Broadcast_op) ->Vec<Input> {

      let secp=&self.0.secp;
      let wallet_key=self.0.create_wallet(self.wallet_purpose(),self.0.recieve,self.0.change);

      let (signer_pub_k,(signer_finger_p, signer_dp))=wallet_key.clone();

      let ext_prv=ExtendedPrivKey::new_master(NETWORK, &self.0.seed).unwrap().derive_priv(&secp, &signer_dp).unwrap();

      let input_list:Vec<Input>=previous_tx.clone().iter().enumerate().map(|(i, previous_tx)|{
      let tweaked_key_pair=ext_prv.to_keypair(&secp).tap_tweak(&secp,None).into_inner();       
      
      let tx_output:Vec<TxOut>= previous_tx.output.clone().iter()
        .filter(|tx_out|UnlockAndSend::new(self,wallet_key.clone()).find_relevent_utxo(tx_out)).map(|tx_out|tx_out.clone()).collect();

      let tap_leaf_hash_list=tx_output.clone().iter().map(|f|TapLeafHash::from_script(&f.script_pubkey, TapScript)).collect::<Vec<TapLeafHash>>();

      let uxto=&tx_output.clone()[0];

      let mut tap_key_origin=BTreeMap::new();

      tap_key_origin.insert
      (signer_pub_k.to_x_only_pub(),
      (tap_leaf_hash_list,(signer_finger_p.clone(),signer_dp.clone())));

      let sig_hash=SighashCache::new(&mut current_tx.clone())
                .taproot_key_spend_signature_hash( i, &Prevouts::All(&previous_tx.output), SchnorrSighashType::AllPlusAnyoneCanPay).unwrap();
      let msg=Message::from_slice(&sig_hash).unwrap();

      let signed_shnorr=secp.sign_schnorr(&msg, &tweaked_key_pair);

      let schnorr_sig=SchnorrSig{sig:signed_shnorr, hash_ty:SchnorrSighashType::AllPlusAnyoneCanPay};
      let mut new_input=Input::default() ;
      new_input.witness_utxo=Some(uxto.clone());
      new_input.tap_key_origins=tap_key_origin;
      new_input.tap_internal_key=Some(signer_pub_k.to_x_only_pub());
      new_input.tap_key_sig=Some(schnorr_sig.clone());
      new_input.non_witness_utxo=Some(previous_tx.clone());
      if(broadcast_op.eq(&Broadcast_op::Finalize)){
        secp.verify_schnorr(&signed_shnorr, &msg, &XOnlyPublicKey::from_slice(&uxto.script_pubkey[2..]).unwrap()).is_ok();
      }
        return new_input;
      }).collect();
      return input_list;
    }
 
  }

impl P2TR {
  pub fn aggregate(&self,address_list:Vec<String>)->String{
      let wallet_key=self.0.create_wallet(self.wallet_purpose(), self.0.recieve, self.0.change);
      let (signer_pub_k,(signer_finger_p, signer_dp))=wallet_key.clone();
      let secp=self.0.secp.clone();

      return address_list.iter().map(|address|{
        let addr=Address::from_str(address).unwrap();
        let x_only_pub_k=signer_pub_k.public_key.to_public_key().inner.combine(&XOnlyPublicKey::from_slice(&addr.script_pubkey()[2..])
        .unwrap().to_public_key().inner).unwrap().to_x_only_pubkey();
        let address=Address::p2tr(&secp, x_only_pub_k,  None, NETWORK);
        return address.to_qr_uri().to_lowercase();
      }).last().unwrap();
  }
}


