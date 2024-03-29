use std::str::FromStr;

use bitcoin::{Address, TxOut};

use crate::bitcoin_wallet::input_data::RpcCall;

pub mod freelancer;
pub mod p2tr_key;
pub mod p2tr_script;
pub mod p2wpkh;
pub mod p2wsh;

pub struct SendToImpl {}

pub trait Wallet<'a, R> {
    fn new(secret_string: Option<&str>, client: &'a R) -> Self
    where
        R: RpcCall;
}

pub fn single_output() -> Box<dyn Fn(u64) -> Vec<TxOut>> {
    return Box::new(|total| {
        let out_put = vec![TxOut {
            value: total,
            script_pubkey: Address::from_str(
                "bcrt1p3z3qayjd0ss65gm660el84j896af9zmcmxg0u7fd2rkx4an2ykzqe7s5c7",
            )
            .unwrap()
            .script_pubkey(),
        }];
        out_put
    });
}

pub fn single_output_with_value(string: String) -> Box<dyn Fn(u64) -> Vec<TxOut>> {
    return Box::new(move |total| {
        let out_put = vec![TxOut {
            value: total,
            script_pubkey: Address::from_str(&string.to_string())
                .unwrap()
                .script_pubkey(),
        }];
        out_put
    });
}

pub fn bisq_output(trade: String, bond: String) -> Box<dyn Fn(u64) -> Vec<TxOut>> {
    return Box::new(move |total| {
        // 3/5
        let trade_total = (total * 3) / 5;
        let out_put = vec![
            TxOut {
                value: (total - trade_total),
                script_pubkey: Address::from_str(&trade.to_string())
                    .unwrap()
                    .script_pubkey(),
            },
            TxOut {
                value: trade_total,
                script_pubkey: Address::from_str(&bond.to_string())
                    .unwrap()
                    .script_pubkey(),
            },
        ];
        out_put
    });
}
