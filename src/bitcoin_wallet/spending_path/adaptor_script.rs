use bitcoin::{secp256k1::{Secp256k1, All, rand::{ RngCore, rngs::OsRng}, SecretKey}, XOnlyPublicKey, Address, Transaction, TxOut, util::taproot::TaprootBuilder, Script, KeyPair, psbt::PartiallySignedTransaction, Witness};

use crate::{bitcoin_wallet::{script_services::{psbt_factory::{LockFn, CreateTxFn, UnlockFn}, output_service::{new_tap_internal_key, insert_tap_tree, insert_tap_key_origin, insert_tree_witness}, input_service::{insert_control_block, sign_tapleaf, sign_2_of_2}}, constants::{Seed, NETWORK, TIP}}, wallet_test::wallet_test_vector_traits::Auxiliary};

use super::scripts::TapScripts;

pub struct AdaptorScript<'a> {
    secp: &'a Secp256k1<All>,
}

impl <'a> AdaptorScript<'a>{

	pub fn new(secp:&'a Secp256k1<All>)->Self{
		return AdaptorScript{secp};
	}

	pub fn adaptor_script(
		&'a self,
		xinternal:XOnlyPublicKey,
		primary_xonly:XOnlyPublicKey,
		secondary_xonly: XOnlyPublicKey
	)->Vec<LockFn<'a>>{
		let delay=TapScripts::delay(&primary_xonly);
		
		let multi_sig=TapScripts::multi_2_of_2_script(&primary_xonly, &secondary_xonly);
		let combined_script = vec![(1, delay.get_script()), (1, multi_sig.get_script())];
		return vec![
			new_tap_internal_key(xinternal),
			insert_tap_tree(combined_script.clone()),
			insert_tap_key_origin(combined_script, primary_xonly),
			insert_tree_witness(&self.secp)
			];
	}

	pub fn generate_auxiliary(auxiliary:Option<Seed>)->Seed{
		return auxiliary.unwrap_or(SecretKey::new(&mut OsRng::new().unwrap()).secret_bytes());
	}

	pub fn adaptor_sig(
		&'a self, 
		untweaked_internal:&'a XOnlyPublicKey,
        key_pair: &'a KeyPair, 
		x_only_2:&'a XOnlyPublicKey,
		auxiliary:&'a [u8; 32]
		
	)->Box<dyn Fn(Vec<Transaction>, Transaction) -> Vec<UnlockFn<'a>> + 'a>
	{
		let x_only=key_pair.public_key();
		let multi_sig=TapScripts::multi_2_of_2_script(&x_only, x_only_2);
		let delay=TapScripts::delay(&x_only);
		let script_weights=vec![(1, multi_sig.get_script()),(1, delay.get_script())];
		let tap_builder=TaprootBuilder::with_huffman_tree(script_weights).unwrap();
		let tap_spending_info=tap_builder.finalize(self.secp,untweaked_internal.clone() ).unwrap();
		let witness=Script::new_v1_p2tr_tweaked(tap_spending_info.output_key());
		return Box::new( move |previous_list, current_tx|{

                let mut unlock_vec: Vec<UnlockFn> = vec![];

                for (size, prev) in previous_list.iter().enumerate() {
					
					unlock_vec.push(
						insert_control_block(
							&self.secp, 
							multi_sig.get_script(), 
							tap_spending_info.clone())
						);

					unlock_vec.push(
						sign_2_of_2(
							&self.secp, 
							current_tx.clone(),
							prev.clone().output,
							size, 
							key_pair,
							witness.clone(),
							multi_sig.get_script(), 
							auxiliary)
						);
				}

				return unlock_vec;
		});
	}

	pub fn merge_psbt(psbt:PartiallySignedTransaction, psbt2:PartiallySignedTransaction)->PartiallySignedTransaction{
		let mut updated_psbt=psbt.clone();
		let mut updated_psbt2=psbt2.clone();
		updated_psbt.inputs.append(&mut updated_psbt2.inputs);
		updated_psbt.outputs.append(&mut updated_psbt2.outputs);
		return updated_psbt;
	}

	
    pub fn finialize_script(
        psbt: PartiallySignedTransaction,
    ) -> Transaction {
        let mut witness = Witness::new();
		psbt.inputs.iter().flat_map(|f|f.tap_script_sigs.iter()
		.map(|sig|sig.1)).for_each(|shnor|{
			witness.push(shnor.to_vec());
		});

		for entry in psbt.inputs[0].clone().tap_scripts{
			let script:Script=entry.1.0;
            witness.push(script);
            witness.push(entry.0.serialize());
		}

        let mut tx = psbt.extract_tx();
        tx.input[0].witness = witness;
        return tx;
    }
}
