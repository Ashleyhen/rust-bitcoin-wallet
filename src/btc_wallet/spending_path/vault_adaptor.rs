use bitcoin::{
    psbt::{Input, Output},
    Transaction, TxIn,
};

use super::Vault;

pub struct VaultAdapter<'l, 'u, L: Vault, U: Vault> {
    lock: &'l L,
    unlock: &'u U,
}

impl<'l, 'u, L: Vault, U: Vault> Vault for VaultAdapter<'l, 'u, L, U> {
    fn create_tx(&self, output_list: &Vec<Output>, tx_in: Vec<TxIn>, total: u64) -> Transaction {
        return self.lock.create_tx(output_list, tx_in, total);
    }

    fn lock_key(&self) -> Vec<Output> {
        return self.lock.lock_key();
    }

    fn unlock_key(&self, previous: Vec<Transaction>, current_tx: &Transaction) -> Vec<Input> {
        return self.unlock.unlock_key(previous, current_tx);
    }
}
impl<'l, 'u, L: Vault, U: Vault> VaultAdapter<'l, 'u, L, U> {
    pub fn new(lock: &'l L, unlock: &'u U) -> Self {
        return VaultAdapter { lock, unlock };
    }
}
