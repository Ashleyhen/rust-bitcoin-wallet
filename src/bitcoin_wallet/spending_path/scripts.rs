use bitcoin::{
    blockdata::{opcodes::all, script::Builder},
    Script, XOnlyPublicKey,
};

pub struct TapScripts {
    script: Script,
}
impl TapScripts {
    pub fn get_script(&self) -> Script {
        return self.script.clone();
    }
    // revocation_script
    pub fn check_single_sig(x_only: &XOnlyPublicKey) -> Self {
        let bob_script = Builder::new()
            .push_x_only_key(&x_only)
            .push_opcode(all::OP_CHECKSIG)
            .into_script();
        return TapScripts { script: bob_script };
    }

    // local_delayedsig
    pub fn delay(x_only: &XOnlyPublicKey) -> Self {
        let bob_script = Builder::new()
            .push_int(144)
            .push_opcode(all::OP_CSV)
            .push_opcode(all::OP_DROP)
            .push_x_only_key(&x_only)
            .push_opcode(all::OP_CHECKSIG)
            .into_script();
        return TapScripts { script: bob_script };
    }

    // https://github.com/Xekyo/murch.one/blob/243b052aad05531f7afcedee45dde1a7594248f6/content/posts/2-of-3-using-P2TR.md
    pub fn multi_2_of_2_script(x_only: &XOnlyPublicKey, x_only2: &XOnlyPublicKey) -> Self {
        let script = Builder::new()
            .push_x_only_key(x_only)
            .push_opcode(all::OP_CHECKSIG)
            .push_x_only_key(x_only2)
            .push_opcode(all::OP_CHECKSIGADD)
            .push_int(2)
            .push_opcode(all::OP_EQUAL)
            .into_script();
        return TapScripts { script };
    }
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
