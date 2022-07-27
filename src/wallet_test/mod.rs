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
use bitcoin::{TxOut, Script, hashes::hex::FromHex, util::taproot::{ScriptLeaf, TapLeafHash, LeafVersion, TapLeafTag}, Address};
use serde::Deserialize;

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
        self.clone().key_spending_path.iter().for_each(|f|{
        f.given.utxos_spent.iter().for_each(|k|{
    dbg!(k.get_tx_out());


    });
        });
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
        return self.bip350address.as_ref().map(|f|Address::from_str(&f).ok()).flatten();
    }

    pub fn get_witness(&self)->Vec<Script>{
        return self.witness.iter().map(|w|Script::from_hex(w).unwrap()).collect::<Vec<Script>>();

    }

    pub fn get_script_patch_control_block(&self)->Vec<Script>{
        return self.script_path_control_blocks.iter().map(|w|Script::from_hex(w).unwrap()).collect::<Vec<Script>>();

    }
}