use std::{str::FromStr, collections::BTreeMap, io::Read};

use bitcoin::{util::{bip32::{DerivationPath, ExtendedPubKey, ExtendedPrivKey, KeySource}, sighash::SighashCache}, psbt::{Output, Input}, Address, Transaction, TxOut, Script, blockdata::{script::Builder, opcodes}, secp256k1::{SecretKey, Message}, EcdsaSig, EcdsaSighashType, Sighash};
use miniscript::ToPublicKey;


use super::{ClientWallet, AddressSchema,  NETWORK,  WalletKeys, ClientWithSchema};

pub struct P2PWKh(ClientWallet) ;

impl AddressSchema for P2PWKh{

    fn map_ext_keys(&self,recieve:&ExtendedPubKey) -> bitcoin::Address {
        return Address::p2wpkh(&recieve.public_key.to_public_key(), NETWORK).unwrap();
    }

    fn new(seed: Option<String>)->Self {
        return P2PWKh(ClientWallet::new(seed));
    }

    fn to_wallet(&self)->ClientWallet {
        return self.0.clone();
    }
 
    fn create_inputs(&self,ext_pub:&ExtendedPubKey,previous_tx:&Transaction) -> Input {
        let mut input_tx=Input::default();
        input_tx.non_witness_utxo=Some((*previous_tx).clone());
        input_tx.witness_utxo=Some(previous_tx.output.iter()
        .filter(|w|w.script_pubkey.eq(&Script::new_v0_p2wpkh(&ext_pub.public_key.to_public_key().wpubkey_hash().unwrap()))).next().unwrap().clone());
        return input_tx;
    }

    fn create_sighash(&self,transaction:&mut bitcoin::Transaction,i:usize,input:&Input,path:&DerivationPath)->EcdsaSig {
        let secp=&self.0.secp;
        let ext_prv=ExtendedPrivKey::new_master(NETWORK, &self.0.seed).unwrap().derive_priv(&secp, path).unwrap();
        
        let sighash=input.witness_utxo.as_ref().map(|tx|{
            return SighashCache::new( transaction).segwit_signature_hash(
                i, &p2wpkh_script_code(&tx.script_pubkey), tx.value, EcdsaSighashType::All)
        }).unwrap().unwrap();
return EcdsaSig::sighash_all(secp.sign_ecdsa(&Message::from_slice(&sighash).unwrap(),&ext_prv.private_key));
    }

    fn wallet_purpose(&self)-> u32 {
        return 84;
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
        


        
// ClientWallet::new(None).
        // path(||)
        // return derivation_path(84, Some(2), 9);
    }