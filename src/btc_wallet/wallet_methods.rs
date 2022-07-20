use std::{collections::BTreeMap, str::FromStr, sync::Arc};

use bdk::KeychainKind;
use bitcoin::{
    psbt::{Input, PartiallySignedTransaction},
    secp256k1::{constants, rand::rngs::OsRng, All, Secp256k1, SecretKey},
    util::bip32::{ChildNumber, DerivationPath, ExtendedPrivKey, ExtendedPubKey},
    Network, OutPoint, Script, Transaction, TxIn, TxOut, Witness,
};

use super::{
    address_formats::AddressSchema,
    constants::{Seed, NETWORK},
    input_data::ApiCall,
    spending_path::Vault,
    WalletKeys,
};

#[derive(Clone)]
pub struct ClientWallet {
    pub secp: Secp256k1<All>,
    pub seed: Seed,
    pub recieve: u32,
    pub change: u32,
}

#[derive(Clone)]
pub struct ClientWithSchema<'a, S: AddressSchema, A: ApiCall> {
    pub schema: &'a S,
    pub api_call: Arc<A>,
}

impl<'a, S: AddressSchema, A: ApiCall> ClientWithSchema<'a, S, A> {
    pub fn new(schema: &S, api_call: A) -> ClientWithSchema<S, A> {
        return ClientWithSchema {
            schema,
            api_call: Arc::new(api_call),
        };
    }

    pub fn get_balance(&self) -> electrum_client::GetBalanceRes {
        let address = self.schema.map_ext_keys(&self.get_ext_pub_k());
        return self
            .api_call
            .script_get_balance(&address.script_pubkey())
            .unwrap();
    }

    pub fn get_ext_pub_k(&self) -> ExtendedPubKey {
        let wallet = self.schema.to_wallet();
        let (ext_pub, _) = self.schema.to_wallet().create_wallet(
            self.schema.wallet_purpose(),
            wallet.recieve,
            wallet.change,
        );
        return ext_pub;
    }

    pub fn print_balance(&self) {
        let get_balance = self.get_balance();
        let address = self.schema.map_ext_keys(&self.get_ext_pub_k());
        println!("address: {}", address);
        println!("confirmed: {}", get_balance.confirmed);
        println!("unconfirmed: {}", get_balance.unconfirmed)
    }

    pub fn change_addr(&self) -> WalletKeys {
        let wallet = self.schema.to_wallet();
        return wallet.create_wallet(
            self.schema.wallet_purpose(),
            wallet.recieve,
            wallet.change + 1,
        );
    }
    pub fn submit_tx<'b, V>(&self, vault: &'b V) -> PartiallySignedTransaction
    where
        V: Vault,
    {
        let wallet = self.schema.to_wallet();
        let signer =
            wallet.create_wallet(self.schema.wallet_purpose(), wallet.recieve, wallet.change);
        let (signer_pub_k, (signer_finger_p, signer_dp)) = signer.clone();

        let signer_addr = self.schema.map_ext_keys(&signer_pub_k);

        let history = Arc::new(
            self.api_call
                .script_list_unspent(&signer_addr.script_pubkey())
                .expect("address history call failed"),
        );

        let tx_in = history
            .clone()
            .iter()
            .map(|tx| {
                return TxIn {
                    previous_output: OutPoint::new(tx.tx_hash, tx.tx_pos.try_into().unwrap()),
                    script_sig: Script::new(), // The scriptSig must be exactly empty or the validation fails (native witness program)
                    sequence: 0xFFFFFFFF,
                    witness: Witness::default(),
                };
            })
            .collect::<Vec<TxIn>>();

        let previous_tx = tx_in
            .iter()
            .map(|tx_id| {
                self.api_call
                    .transaction_get(&tx_id.previous_output.txid)
                    .unwrap()
            })
            .collect::<Vec<Transaction>>();

        let lock_list = vault.lock_key(self.schema);
        let current_tx = vault.create_tx(&lock_list, tx_in, self.get_balance().confirmed);

        let input_vec = vault.unlock_key(previous_tx.to_vec().clone(), &current_tx);

        let mut xpub = BTreeMap::new();

        // the current public key and derivation path
        xpub.insert(signer_pub_k, (signer_finger_p, signer_dp.clone()));

        // we have all the infomation for the partially signed transaction
        let psbt = PartiallySignedTransaction {
            unsigned_tx: current_tx.clone(),
            version: 2,
            xpub,
            proprietary: BTreeMap::new(),
            unknown: BTreeMap::new(),
            outputs: lock_list.iter().map(|output| output.clone()).collect(),
            inputs: input_vec,
        };
        return psbt;
    }
}
impl ClientWallet {
    pub fn new(secret_seed: Option<String>, recieve: u32, change: u32) -> ClientWallet {
        return ClientWallet {
            secp: Secp256k1::new(),
            seed: (match secret_seed {
                Some(secret) => SecretKey::from_str(&secret)
                    .expect("failed to initalize seed, check if the seed is valid"),
                _ => SecretKey::new(
                    &mut OsRng::new().expect("failed to create a random number generator !!! "),
                ),
            })
            .secret_bytes(),
            recieve,
            change,
        };
    }

    pub fn create_wallet(&self, purpose: u32, recieve: u32, index: u32) -> WalletKeys {
        // bip84 For the purpose-path level it uses 84'. The rest of the levels are used as defined in BIP44 or BIP49.
        // m / purpose' / coin_type' / account' / change / address_index
        let keychain = KeychainKind::External;
        let path = DerivationPath::from(vec![
            ChildNumber::from_hardened_idx(purpose).unwrap(), // purpose
            ChildNumber::from_hardened_idx(recieve).unwrap(), // first recieve
            ChildNumber::from_hardened_idx(0).unwrap(),       // second recieve
            ChildNumber::from_normal_idx(keychain as u32).unwrap(),
            ChildNumber::from_normal_idx(index).unwrap(),
        ]);
        let ext_prv = ExtendedPrivKey::new_master(NETWORK, &self.seed)
            .unwrap()
            .derive_priv(&self.secp, &path)
            .unwrap();
        let ext_pub = ExtendedPubKey::from_priv(&self.secp, &ext_prv);

        return (ext_pub, (ext_pub.fingerprint(), path));
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum Broadcast_op {
    Finalize,
    None,
    Broadcast,
}

// fn path<F>(change:F)
//         where F: FnOnce(Script) ->dyn UnlockPreviousUTXO{

// }

// multi sig
