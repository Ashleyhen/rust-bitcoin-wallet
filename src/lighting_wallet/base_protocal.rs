use bitcoin::secp256k1::{SecretKey, rand::rngs::OsRng};
use lightning::ln::{peer_handler::PeerManager, msgs::{Init, NetAddress, ErrorMessage, WarningMessage, Ping, Pong}, features::InitFeatures};

// messages related to set up and control 
pub fn base_protocal(){
    let remote_network_address=NetAddress::IPv4 { addr: [127,0,0,1], port: 8182 };

    let init_msg=Init{ features:InitFeatures::known(), remote_network_address:Some(remote_network_address) };

    let seed=SecretKey::new(&mut OsRng::new().unwrap()).secret_bytes();

    let err_msg=ErrorMessage{ channel_id: seed, data: "error message".to_owned() };
    let warn_msg=WarningMessage{ channel_id: seed, data: "warning message".to_owned() };

    println!("init_msg: {:#?}",init_msg);
    println!("error message: {:#?}",err_msg);
    println!("warning: {:#?}",warn_msg);
    // control message

    let ping = Ping{ ponglen: 32, byteslen: 0 };
    println!("ping: {:#?}",ping);

    let pong=Pong{ byteslen: 32 };
    println!("pong: {:#?}",pong);



}