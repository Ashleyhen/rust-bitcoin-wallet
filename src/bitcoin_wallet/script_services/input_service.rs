use bitcoin::{
    blockdata::{opcodes, script::Builder},
    psbt::Input,
    schnorr::TapTweak,
    secp256k1::{All, Message, Secp256k1},
    util::{
        bip32::ExtendedPrivKey,
        sighash::{Prevouts, ScriptPath, SighashCache},
        taproot::{LeafVersion, TapLeafHash, TaprootSpendInfo},
    },
    Address, EcdsaSig, KeyPair, SchnorrSig, SchnorrSighashType, Script, Transaction, TxOut,
};

use miniscript::ToPublicKey;

use crate::bitcoin_wallet::constants::NETWORK;

pub fn insert_control_block<'a>(
    secp: &'a Secp256k1<All>,
    spending_script: Script,
    spending_info: TaprootSpendInfo,
) -> Box<impl FnOnce(&mut Input) + 'a> {
    return Box::new(move |input: &mut Input| {
        let control =
            spending_info.control_block(&(spending_script.clone(), LeafVersion::TapScript));
        let verify = control.as_ref().unwrap().verify_taproot_commitment(
            &secp,
            spending_info.output_key().to_inner(),
            &spending_script,
        );
        print!("is this control block valid {}", verify);
        input.tap_scripts.insert(
            control.unwrap(),
            (spending_script.clone(), LeafVersion::TapScript),
        );
        let witness = Address::p2tr_tweaked(spending_info.output_key(), NETWORK);

        input.witness_script = Some(witness.script_pubkey());
        input.tap_merkle_root = spending_info.merkle_root();
    });
}

pub fn insert_witness_tx<'a>(tx_out: TxOut) -> Box<impl FnOnce(&mut Input) + 'a> {
    return Box::new(move |input: &mut Input| {
        input.witness_utxo = Some(tx_out);
    });
}

pub fn filter_for_wit(previous_tx: Vec<TxOut>, witness: &Script) -> Vec<TxOut> {
    return previous_tx
        .iter()
        .filter(|t| t.script_pubkey.eq(&witness))
        .map(|a| a.clone())
        .collect::<Vec<TxOut>>();
}


pub fn print_tx_out_addr(addr_list:Vec<(String,&Vec<TxOut>)>)->String{
        let mut dbg_err=String::from("\n");
        addr_list.iter().for_each(|(var_name,tx_list)|{
            dbg_err.push_str(&var_name);
            dbg_err.push_str(": \n");
            tx_list.iter().map(|tx_out|Address::from_script(&tx_out.script_pubkey,NETWORK).unwrap().to_string()).for_each(|addr| { 
                dbg!(addr.clone());
                dbg_err.push_str(&addr);
                dbg_err.push_str("\n");
            });

        });
        dbg_err.push_str("\n");
        return dbg_err.to_string();
    }

pub fn sign_tapleaf<'a>(
    secp: &'a Secp256k1<All>,
    key_pair: &'a KeyPair,
    current_tx: Transaction,
    previous_tx: Vec<TxOut>,
    input_index: usize,
    witness_script: Script,
    bob_script: Script,
) -> Box<impl FnOnce(&mut Input) + 'a> {
    let x_only = key_pair.public_key();
    return Box::new(move |input: &mut Input| {
        let prev = filter_for_wit(previous_tx.clone(), &witness_script);
        

        let tap_leaf_hash = TapLeafHash::from_script(&bob_script, LeafVersion::TapScript);
        
        let tap_sig_hash = SighashCache::new(&current_tx)
            .taproot_script_spend_signature_hash(
                input_index,
                &Prevouts::All(&prev),
                ScriptPath::with_defaults(&bob_script),
                SchnorrSighashType::AllPlusAnyoneCanPay,
            )
            .expect(&print_tx_out_addr(vec![("prevouts ".to_owned(),&previous_tx),("current tx ".to_string(),&current_tx.output)]));

        let sig = secp.sign_schnorr(&Message::from_slice(&tap_sig_hash).unwrap(), &key_pair);
        let schnorrsig = SchnorrSig {
            sig,
            hash_ty: SchnorrSighashType::AllPlusAnyoneCanPay,
        };

        input
            .tap_script_sigs
            .insert((x_only, tap_leaf_hash), schnorrsig);
    });
}

pub fn sign_key_sig<'a>(
    secp: &'a Secp256k1<All>,
    key_pair: &'a KeyPair,
    current_tx: Transaction,
    previous_tx: Vec<TxOut>,
    input_index: usize,
) -> Box<impl FnOnce(&mut Input) + 'a> {
    return Box::new(move |input: &mut Input| {
        let witness_script = input.clone().witness_script.unwrap();
        let prev = filter_for_wit(previous_tx, &witness_script);
        let tap_sig = SighashCache::new(&current_tx)
            .taproot_key_spend_signature_hash(
                input_index,
                &Prevouts::All(&prev),
                SchnorrSighashType::AllPlusAnyoneCanPay,
            )
            .unwrap();
        let tweaked_pair = key_pair.tap_tweak(&secp, input.tap_merkle_root);

        let sig = secp.sign_schnorr(
            &Message::from_slice(&tap_sig).unwrap(),
            &tweaked_pair.into_inner(),
        );
        let schnorrsig = SchnorrSig {
            sig,
            hash_ty: SchnorrSighashType::AllPlusAnyoneCanPay,
        };
        input.tap_key_sig = Some(schnorrsig);
    });
}

pub fn sign_segwit_v0<'a>(
    secp: &'a Secp256k1<All>,
    current_tx: Transaction,
    tx_out: TxOut,
    input_index: usize,
    script_code: Script,
    extended_priv_k: ExtendedPrivKey,
) -> Box<impl FnOnce(&mut Input) + 'a> {
    Box::new(move |input: &mut Input| {
        let public_key = extended_priv_k
            .to_keypair(&secp)
            .public_key()
            .to_public_key();
        let sig_hash = SighashCache::new(&mut current_tx.clone())
            .segwit_signature_hash(
                input_index,
                &script_code,
                tx_out.value,
                bitcoin::EcdsaSighashType::All,
            )
            .unwrap();

        let msg = Message::from_slice(&sig_hash).unwrap();
        let sig = EcdsaSig::sighash_all(secp.sign_ecdsa(&msg, &extended_priv_k.private_key));

        input.partial_sigs.insert(public_key, sig);
    })
}

pub fn p2wpkh_script_code(script: &Script) -> Script {
    Builder::new()
        .push_opcode(opcodes::all::OP_DUP)
        .push_opcode(opcodes::all::OP_HASH160)
        .push_slice(&script[2..])
        .push_opcode(opcodes::all::OP_EQUALVERIFY)
        .push_opcode(opcodes::all::OP_CHECKSIG)
        .into_script()
}
