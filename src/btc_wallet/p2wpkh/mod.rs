use std::{str::FromStr, collections::BTreeMap, io::Read, sync::Arc};

use bdk::template::P2Pkh;
use bitcoin::{util::{bip32::{DerivationPath, ExtendedPubKey, ExtendedPrivKey, KeySource}, sighash::SighashCache}, psbt::{Output, Input}, Address, Transaction, TxOut, Script, blockdata::{script::Builder, opcodes}, secp256k1::{SecretKey, Message}, EcdsaSig, EcdsaSighashType, Sighash};
use miniscript::ToPublicKey;


use super::{ClientWallet, AddressSchema,  NETWORK,  WalletKeys, ClientWithSchema, UnlockPreviousUTXO};

pub struct P2PWKh(pub ClientWallet) ;

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

    fn wallet_purpose(&self)-> u32 {
        return 84;
    }
}

impl UnlockPreviousUTXO for P2PWKh {
    fn prv_tx_input(&self,ext_pub:&ExtendedPubKey,previous_tx:&Transaction) -> Input {
        let mut input_tx=Input::default();
        input_tx.non_witness_utxo=Some((*previous_tx).clone());
        input_tx.witness_utxo=Some(previous_tx.output.iter()
        .filter(|w|w.script_pubkey.eq(&Script::new_v0_p2wpkh(&ext_pub.public_key.to_public_key().wpubkey_hash().unwrap()))).next().unwrap().clone());
        return input_tx;
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