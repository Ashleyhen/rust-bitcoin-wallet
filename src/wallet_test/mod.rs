pub mod wallet_test_vector_traits;
use std::{
    fs::{self, File},
    io::{BufRead, BufReader},
    sync::Arc,
};

use crate::wallet_test::wallet_test_vector_traits::{
    Auxiliary, Given, ScriptPubKey, ScriptTree, WalletTestVectors,
};
use bincode::{config::Infinite, deserialize, serialize};
use serde::Deserialize;

pub fn printOutTest() {
    let foobar: Arc<WalletTestVectors> = Arc::new(
        serde_json::from_reader(BufReader::new(
            File::open("src/wallet_test/wallet-test-vectors.json").unwrap(),
        ))
        .unwrap(),
    );
    dbg!(foobar);
}
