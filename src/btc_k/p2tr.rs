use bitcoin::Address;

use super::{AddressSchema, ClientWallet, NETWORK};

pub struct P2TR(ClientWallet);

impl AddressSchema for P2TR{
    fn map_ext_keys(&self,recieve:&bitcoin::util::bip32::ExtendedPubKey) -> bitcoin::Address {
		return Address::p2tr(&self.0.secp, recieve.to_x_only_pub(), None, NETWORK);
    }

    fn create_inputs(&self,wallet_keys:&bitcoin::util::bip32::ExtendedPubKey,previous_tx:&bitcoin::Transaction ) -> bitcoin::psbt::Input {
        todo!()
    }

    // fn new_wallet(&self,recieve:u32,change:u32)-> super::WalletKeys {
    //     return self.0.create_wallet(341, recieve, change);
    // }

    fn new(seed: Option<String>)->Self {
		return P2TR(ClientWallet::new(seed));
    }

    fn to_wallet(&self)->ClientWallet {
		return self.0.clone();
    }

    fn create_sighash(&self,cache:&mut bitcoin::Transaction,s:usize,input:&bitcoin::psbt::Input,dp:&bitcoin::util::bip32::DerivationPath)->bitcoin::EcdsaSig {
        todo!()
    }

    fn wallet_purpose(&self)-> u32 {
        return 341;
    }
}