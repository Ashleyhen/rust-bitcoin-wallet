use std::{collections::BTreeMap, borrow::Borrow};

use bitcoin::{Address, util::{bip32::{DerivationPath, ExtendedPubKey, ExtendedPrivKey, KeySource}, sighash::{SighashCache, Prevouts}, taproot::{TapLeafHash,LeafVersion::TapScript}}, Transaction, psbt::Input, Script, SchnorrSighashType, SchnorrSig, secp256k1::{schnorr::Signature, Message, schnorrsig::PublicKey }, KeyPair, TxOut, blockdata::{script::Builder, opcodes},  XOnlyPublicKey, schnorr::{UntweakedPublicKey, TweakedPublicKey}};
use miniscript::interpreter::KeySigPair;

use super::{AddressSchema, ClientWallet, NETWORK, UnlockPreviousUTXO, WalletKeys};

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
}
impl UnlockPreviousUTXO for P2TR{
    fn prv_tx_input(&self,ext_pub:&ExtendedPubKey,previous_tx:&bitcoin::Transaction ) ->bitcoin::psbt::Input {

      let mut input_tx= Input::default();
      let tr_script=Script::new_v1_p2tr(&self.0.secp,ext_pub.to_x_only_pub(),None);
      input_tx.non_witness_utxo=Some((*previous_tx).clone());
      
      input_tx.witness_utxo=Some(previous_tx.output.iter()
      .filter(|w|w.script_pubkey.eq(&tr_script)).next().unwrap().clone());
// input_tx.witness_utxo.unwrap().script_pubkey

// Script::new_v1_p2tr(&secp, key_pair.public_key(), None);
      return input_tx;
    }

    fn prv_psbt_input(&self,prev_transaction:&mut Transaction,input_outpoints: &Input,i:usize,wallet_keys:&WalletKeys)->Input {
      let (signer_pub_k,(signer_finger_p, signer_dp))=wallet_keys;
      let secp=&self.0.secp;
      let ext_prv=ExtendedPrivKey::new_master(NETWORK, &self.0.seed).unwrap().derive_priv(&secp, &signer_dp).unwrap();


      let out_tx_list=prev_transaction.output.iter().filter(|out|
        Address::p2tr(secp, signer_pub_k.to_x_only_pub(), None, NETWORK)
        .eq(&Address::from_script(&out.script_pubkey,NETWORK).unwrap())).map(|out|out.clone())
        .map(|f|{
          
return TxOut{
        value: f.value,
        // script_pubkey: Script::new_v1_p2tr(&secp, signer_pub_k.to_x_only_pub(), None),
        script_pubkey: Script::new_v1_p2tr_tweaked( TweakedPublicKey::dangerous_assume_tweaked(
          XOnlyPublicKey::from_slice(&f.script_pubkey[2..]).unwrap()))

    };
        })
        .collect::<Vec<TxOut>>()
        ;
   let temp=Script::new_v1_p2tr_tweaked( TweakedPublicKey::dangerous_assume_tweaked(signer_pub_k.to_x_only_pub()));
    let tx_out=TxOut{
        value: input_outpoints.clone().witness_utxo.unwrap().value,
        // script_pubkey: Script::new_v1_p2tr(&secp, signer_pub_k.to_x_only_pub(), None),
        script_pubkey:temp
    };
    
        
        let mut tap_key_origin=BTreeMap::new();
let script_list=out_tx_list.clone().iter().map(|f|TapLeafHash::from_script(&f.script_pubkey, TapScript)).collect::<Vec<TapLeafHash>>();
        tap_key_origin.insert
        (signer_pub_k.to_x_only_pub(),
        (script_list,(signer_finger_p.clone(),signer_dp.clone())));

        // Interpreter::verify_sig(&self, secp, tx, input_idx, prevouts, sig) ;
        // PublicKey::f
        
        // out_tx.iter().map(|f|Address::from_script(&f.script_pubkey, NETWORK).unwrap());
        
      let mut new_input=input_outpoints.clone() ;
// let a=*out_tx_list.clone().iter().next().unwrap();
    new_input.witness_utxo=Some(out_tx_list.iter().next().map(|t|t.clone()).unwrap());
      let sighash=SighashCache::new(&mut prev_transaction.clone()).taproot_key_spend_signature_hash(
              i, &Prevouts::All(&out_tx_list), SchnorrSighashType::Default).unwrap();
              
let msg=Message::from_slice(&sighash).unwrap();
let key_pair=&ext_prv.to_keypair(secp);

let signed_shnorr=secp.sign_schnorr(&msg, &key_pair);

let schnorr_sig=SchnorrSig{ sig:signed_shnorr ,hash_ty: SchnorrSighashType::Default };

let key=KeySigPair::Schnorr(signer_pub_k.to_x_only_pub(),schnorr_sig );

let is_successful=secp.verify_schnorr(&signed_shnorr, &msg, &key_pair.public_key()).is_ok();

// BitcoinKey;
            new_input.tap_internal_key=Some(signer_pub_k.to_x_only_pub());
      new_input.tap_key_sig=Some(schnorr_sig.clone());
      
      // dbg!(x_only);
    //   new_input.tap_key_origins=tap_key_origin;

   
    
      let x_only_2=XOnlyPublicKey::from_slice(&tx_out.script_pubkey[2..]).unwrap();

      let x_only=XOnlyPublicKey::from_slice(&new_input.clone().witness_utxo.unwrap().script_pubkey[2..]).unwrap();
    
let x=signer_pub_k.to_x_only_pub();
    self.0.secp.verify_schnorr(&signed_shnorr, &msg, &x_only_2).unwrap();
      // [29] = 244
      // [30] = 250
      // [31] = 230
    // println!("signed_shnorr");
    // [29] = 26
    //   [30] = 145
    //   [31] = 149
      // dbg!(x_only_temp);
      // let x_only_k_bin=[24 ,112 ,254 ,86 ,102 ,216 ,167 ,44 ,199 ,220 ,136 ,204 ,122 ,79 ,212 ,242 ,121 ,114 ,118 ,4 ,93 ,62 ,251 ,50 ,108 ,57 ,106 ,7 ,246 ,244 ,250 ,230 ,152 ,244 ,182 ,108 ,88 ,228 ,27 ,50 ,113 ,242 ,158 ,152 ,167 ,168 ,188 ,196 ,36 ,115 ,69 ,42 ,2 ,215 ,77 ,154 ,70 ,189 ,22 ,120 ,0 ,15 ,144 ,169];
// let x_only_temp_2=XOnlyPublicKey::from_slice(&x_only_k_bin).unwrap();
// dbg!(x_only_temp_2);
      // let addr=Address::p2tr(secp, x_only_temp_2, None, NETWORK).to_qr_uri().to_lowercase();
      // println!("{}", addr);
      
 
      return new_input;

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
