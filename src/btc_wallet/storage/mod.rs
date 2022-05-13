
use std::{fs::File, u8};

use bitcoin::util::bip32::{DerivationPath, ChildNumber};
use serde::{Deserialize, Serialize, ser::SerializeStruct, Serializer};

use super::{ClientWallet, Seed};

#[derive(Serialize, Deserialize, Debug)]
pub struct UxtoStorage {
    seed:Seed,
    derivation_paths:Vec<ChildNumber>,
}

 impl UxtoStorage{
    pub fn new(seed:Seed,dp:Vec<DerivationPath> )->UxtoStorage{

        return UxtoStorage{
            seed,
            derivation_paths: dp.iter().flat_map(|f|f.into_iter().map(|a |(*a))).collect::<Vec<ChildNumber>>()
        };
    }
    pub fn save(&self){
        let mut writer=File::create("../spending.bin").unwrap();
        bincode::serialize_into(writer, self).unwrap();

    }

}   
