pub mod rpc_call;

use bitcoin::secp256k1::{rand::rngs::OsRng, Scalar, SecretKey};
use lightning::ln::{
    features::InitFeatures,
    msgs::{ErrorMessage, Init, NetAddress, OpenChannel, Ping, Pong, WarningMessage},
    peer_handler::PeerManager,
};

// messages related to set up and control
pub fn base_protocal() {
    let remote_network_address = NetAddress::IPv4 {
        addr: [127, 0, 0, 1],
        port: 8182,
    };

    let init_msg = Init {
        features: InitFeatures::empty(),
        remote_network_address: Some(remote_network_address),
    };

    let seed = Scalar::random().to_be_bytes();

    // OpenChannel
    let err_msg = ErrorMessage {
        channel_id: seed,
        data: "error message".to_owned(),
    };
    let warn_msg = WarningMessage {
        channel_id: seed,
        data: "warning message".to_owned(),
    };

    println!("init_msg: {:#?}", init_msg);
    println!("error message: {:#?}", err_msg);
    println!("warning: {:#?}", warn_msg);
    // control message

    let ping = Ping {
        ponglen: 32,
        byteslen: 0,
    };
    println!("ping: {:#?}", ping);

    let pong = Pong { byteslen: 32 };
    println!("pong: {:#?}", pong);
}
pub fn testrpc() {}
