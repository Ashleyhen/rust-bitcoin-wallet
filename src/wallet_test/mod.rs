pub mod wallet_test_vector_traits;
use std::{
    fs::{self, File},
    io::{BufRead, BufReader},
    sync::Arc, str::FromStr,
};

use crate::wallet_test::wallet_test_vector_traits::{
    Auxiliary, Given,  ScriptTree, WalletTestVectors,
};
use bincode::{config::Infinite, deserialize, serialize};
use bitcoin::{TxOut, Script, hashes::hex::{FromHex, HexIterator}, util::{taproot::{ScriptLeaf, TapLeafHash, LeafVersion, TapLeafTag, ControlBlock, TapSighashHash, TapSighashTag}, sighash::ScriptPath, bip32::ExtendedPrivKey}, Address, SigHashType, SchnorrSighashType, Transaction, psbt::serialize::Deserialize, schnorr::KeyPair, secp256k1::{PublicKey, SecretKey}, XOnlyPublicKey, PrivateKey,  };

use self::wallet_test_vector_traits::{UtxosSpent, Expected};

impl WalletTestVectors{
    pub fn load_test()->Self {
        return 
            serde_json::from_reader(BufReader::new(
                File::open("src/wallet_test/wallet-test-vectors.json").unwrap(),
            ))
            .unwrap();
        
        // dbg!(foobar) ;
    }

    pub fn test(&self){

        self.clone().key_spending_path.iter().for_each(|f|
         {
             
        f.input_spending.iter().for_each(|i|{
        
            dbg!(i.given.get_internal_privkey());
        });
// f.given.get_utxos_spent().iter().for_each(|f|{
//                 dbg!(f);
//             });
// /**/  
dbg!(f.given.get_internal_privkey());
         } ); 
    }
    
}

impl UtxosSpent {
    pub fn get_tx_out(&self)->TxOut{
        return TxOut{ value:self.amount_sats, script_pubkey:Script::from_hex(&self.script_pub_key).unwrap()};
    }
}

impl ScriptTree{
    pub fn get_script_leaf(&self)->TapLeafHash{
        return TapLeafHash::from_script(&Script::from_hex(&self.script).unwrap(), LeafVersion::TapScript);
    }
}
impl Expected{

    pub fn get_bip_350_address(&self)->Option<Address>{
        return self.bip350address.as_ref().map(|f|Address::from_str(&f).unwrap());
    }

    pub fn get_witness(&self)->Vec<Script>{
        return self.witness.iter().map(|w|Script::from_hex(w).unwrap()).collect::<Vec<Script>>();

    }

    pub fn get_script_patch_control_block(&self)->Vec<ControlBlock>{
         return self.script_path_control_blocks.iter().map(|w|
            ControlBlock::from_slice(&Vec::from_hex(w).unwrap().clone()).unwrap()
        ).collect::<Vec<ControlBlock>>();
    }
}

impl Given{
        pub fn get_merkle_root(&self){}
        pub fn get_script_tree(&self){}
        pub fn get_txin_index(&self){}

        pub fn get_utxos_spent(&self)->Vec<TxOut>{
            return self.utxos_spent.iter().map(|f|f.get_tx_out()).collect::<Vec<TxOut>>();
        }

        pub fn get_hash(&self)->Option<SchnorrSighashType>{
            return self.hash_type.map(|f|SchnorrSighashType::from_u8(f).unwrap());
        }
        

        pub fn get_unsigned_tx(&self)->Option<Transaction>{
            return self.raw_unsigned_tx.as_ref().map(|w|
                Transaction::deserialize(&Vec::from_hex(w).unwrap()).unwrap()
            );
        }

        pub fn get_internal_pubkey(&self)->Option<XOnlyPublicKey>{
            return self.internal_pubkey.as_ref().map(|f| XOnlyPublicKey::from_str(f).unwrap());
        }
        pub fn get_internal_privkey(&self)->Option<SecretKey>{
            return self.internal_privkey.as_ref().map(|f|SecretKey::from_str(f).unwrap())
        }

}