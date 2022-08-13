use bitcoin::{
    psbt::{Input, Output},
    schnorr::TapTweak,
    secp256k1::Message,
    util::{
        sighash::{Prevouts, SighashCache},
        taproot::{LeafVersion, TaprootSpendInfo},
    },
    SchnorrSig, SchnorrSighashType, Script, Transaction, TxIn, TxOut,
};

use crate::btc_wallet::address_formats::{p2tr_addr_fmt::P2TR, AddressSchema};

pub struct InputService(pub P2TR);

impl InputService {
    pub fn insert_givens<'a>(&'a self) -> Box<impl FnOnce(&Output, &mut Input) + 'a> {
        return Box::new(move |output: &Output, input: &mut Input| {
            let out = output.clone();
            input.witness_script = out.witness_script;
            input.tap_internal_key = out.tap_internal_key;
            input.tap_key_origins = out.tap_key_origins;
        });
    }

    pub fn insert_control_block<'a>(
        &'a self,
        script: Script,
    ) -> Box<impl FnOnce(&Output, &mut Input) + 'a> {
        let secp = self.0.get_client_wallet().secp;
        let x_only = self.0.get_ext_pub_key().to_x_only_pub();
        let mut err_msg = "missing expected scripts for x_only ".to_string();
        err_msg.push_str(&x_only.to_string());

        return Box::new(move |output: &Output, input: &mut Input| {
            let internal_key = input.tap_internal_key.expect("msg");
            let spending_info = TaprootSpendInfo::with_huffman_tree(
                &secp,
                internal_key,
                output
                    .tap_tree
                    .as_ref()
                    .unwrap()
                    .script_leaves()
                    .map(|s| (u32::from(s.depth()), s.script().clone()))
                    .collect::<Vec<(u32, Script)>>(),
            )
            .unwrap();

            let control = spending_info.control_block(&(script.clone(), LeafVersion::TapScript));
            control.as_ref().unwrap().verify_taproot_commitment(
                &secp,
                spending_info.output_key().to_inner(),
                &script,
            );
            input
                .tap_scripts
                .insert(control.unwrap(), (script.clone(), LeafVersion::TapScript));
        });
    }

    pub fn sign_tapleaf<'a>(
        &'a self,
        current_tx: Transaction,
        previous_tx: Vec<TxOut>,
        input_index: usize,
    ) -> Box<impl FnOnce(&Output, &mut Input) + 'a> {
        let secp = self.0.get_client_wallet().secp;
        let x_only = self.0.get_ext_pub_key().to_x_only_pub();
        return Box::new(move |output: &Output, input: &mut Input| {
            let witness_script = output
                .witness_script
                .as_ref()
                .expect("missing witness script");
                dbg!(witness_script);
            let prev = previous_tx
                .iter()
                .filter(|t| t.script_pubkey.eq(&witness_script))
                .map(|a| a.clone())
                .collect::<Vec<TxOut>>();
            let tap_leaf_hash = output
                .tap_key_origins
                .get(&self.0.get_ext_pub_key().to_x_only_pub())
                .unwrap()
                .0
                .clone();
            let tap_sig_hash = SighashCache::new(&current_tx)
                .taproot_script_spend_signature_hash(
                    input_index,
                    &Prevouts::All(&prev),
                    tap_leaf_hash[0],
                    SchnorrSighashType::AllPlusAnyoneCanPay,
                )
                .unwrap();
            let tweaked_pair = self
                .0
                .get_ext_prv_k()
                .to_keypair(&secp)
                .tap_tweak(&secp, input.tap_merkle_root);
            let sig = secp.sign_schnorr(
                &Message::from_slice(&tap_sig_hash).unwrap(),
                &tweaked_pair.into_inner(),
            );
            let schnorrsig = SchnorrSig {
                sig,
                hash_ty: SchnorrSighashType::AllPlusAnyoneCanPay,
            };
            input
                .tap_script_sigs
                .insert((x_only, tap_leaf_hash[0]), schnorrsig);
        });
    }
}
