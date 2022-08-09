use bitcoin::{hashes::hex::FromHex, psbt::serialize::Deserialize, Transaction, TxIn};
use electrum_client::{Error, GetBalanceRes};

use super::RpcCall;

pub struct TapscriptExInput();

impl RpcCall for TapscriptExInput {
    fn contract_source(&self) -> (Vec<TxIn>, Vec<Transaction>) {
        let tx = get_tx();
        return (tx.clone().input, vec![tx]);
    }

    fn script_get_balance(&self) -> Result<GetBalanceRes, Error> {
        return Ok(GetBalanceRes {
            confirmed: get_tx().output.iter().map(|f| f.value).sum(),
            unconfirmed: 0,
        });
    }
}

impl TapscriptExInput {
    pub fn new() -> Self {
        return TapscriptExInput();
    }
}
pub fn tx_as_hash() -> Transaction {
    return Transaction::deserialize(&Vec::from_hex("020000000171f2f89c07c3b58c7b0cf3654ba049d28bbcc76b7298f41c17e7b1a3149040ec0000000000ffffffff01905f010000000000160014ceb2d28afdcad1ae0fc2cf81cb929ba29e83468200000000").unwrap()).unwrap();
}

pub fn get_tx() -> Transaction {
    return Transaction::deserialize(&Vec::from_hex("020000000001010aa633878f200c80fc8ec88f13f746e5870be7373ad5d78d22e14a402d6c6fc20000000000feffffff02a086010000000000225120a5ba0871796eb49fb4caa6bf78e675b9455e2d66e751676420f8381d5dda8951c759f405000000001600147bf84e78c81b9fed7a47b9251d95b13d6ebac14102473044022017de23798d7a01946744421fbb79a48556da809a9ffdb729f6e5983051480991022052460a5082749422804ad2a25e6f8335d5cf31f69799cece4a1ccc0256d5010701210257e0052b0ec6736ee13392940b7932571ce91659f71e899210b8daaf6f17027500000000").unwrap()).unwrap();
}
