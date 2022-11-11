use bitcoin::{
    blockdata::{opcodes::all, script::Builder},
    secp256k1::PublicKey,
    Script, XOnlyPublicKey,
};
use miniscript::ToPublicKey;

// revocation_script
pub fn check_single_sig(x_only: &XOnlyPublicKey) -> Script {
    return Builder::new()
        .push_x_only_key(&x_only)
        .push_opcode(all::OP_CHECKSIG)
        .into_script();
}

// local_delayedsig
pub fn delay(x_only: &XOnlyPublicKey) -> Script {
    return Builder::new()
        .push_int(144)
        .push_opcode(all::OP_CSV)
        .push_opcode(all::OP_DROP)
        .push_x_only_key(&x_only)
        .push_opcode(all::OP_CHECKSIG)
        .into_script();
}

// https://github.com/Xekyo/murch.one/blob/243b052aad05531f7afcedee45dde1a7594248f6/content/posts/2-of-3-using-P2TR.md
pub fn multi_2_of_2_script(x_only: &XOnlyPublicKey, x_only2: &XOnlyPublicKey) -> Script {
    return Builder::new()
        .push_x_only_key(x_only)
        .push_opcode(all::OP_CHECKSIG)
        .push_x_only_key(x_only2)
        .push_opcode(all::OP_CHECKSIGADD)
        .push_int(2)
        .push_opcode(all::OP_EQUAL)
        .into_script();
}

pub fn p2wpkh_script_code(script: &Script) -> Script {
    Builder::new()
        .push_opcode(all::OP_DUP)
        .push_opcode(all::OP_HASH160)
        .push_slice(&script[2..])
        .push_opcode(all::OP_EQUALVERIFY)
        .push_opcode(all::OP_CHECKSIG)
        .into_script()
}

pub fn p2wsh_multi_sig(pub_keys: &Vec<PublicKey>) -> Script {
    fn partial_p2wsh_multi_sig<'a>(mut iter: impl Iterator<Item = &'a PublicKey>) -> Builder {
        match iter.next() {
            Some(pub_k) => partial_p2wsh_multi_sig(iter).push_key(&pub_k.to_public_key()),
            None => Builder::new().push_int(2),
        }
    }

    return partial_p2wsh_multi_sig(pub_keys.iter())
        .push_int(2)
        .push_opcode(all::OP_CHECKMULTISIG)
        .into_script();
}
