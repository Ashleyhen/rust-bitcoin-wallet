use bitcoin::{
    secp256k1::{All, Secp256k1},
    Transaction,
};

use crate::bitcoin_wallet::script_services::psbt_factory::{LockFn, UnlockFn};

pub struct P2wsh {
    pub secp: Secp256k1<All>,
}

impl P2wsh {
    pub fn new(secp: &Secp256k1<All>) -> Self {
        return P2wsh { secp: secp.clone() };
    }

    pub fn input_factory<'a>(
        &'a self,
    ) -> Box<dyn Fn(Vec<Transaction>, Transaction) -> Vec<UnlockFn<'a>> + 'a> {
        return Box::new(
            move |previous_list: Vec<Transaction>, current: Transaction| {
                let mut unlock_vec: Vec<UnlockFn> = vec![];
                for (input_index, prev) in previous_list.iter().enumerate() {}
                return unlock_vec;
            },
        );
    }

    pub fn output_factory<'a>(&'a self) -> Vec<LockFn<'a>> {
        return vec![];
    }
}

//
