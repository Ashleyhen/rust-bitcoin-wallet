use clightningrpc::requests::GetInfo;
use traproot_bdk::{
    connect_lightning,
    lnrpc::{
        ConnectPeerRequest, LightningAddress, ListPeersRequest, NewAddressRequest,
        OpenChannelRequest, GetInfoRequest,
    },
};

use crate::{
    bitcoin_wallet::{constants::SEED, input_data::regtest_call::RegtestCall},
    simple_wallet::{p2tr_key::P2TR, single_output_with_value, Wallet},
};

#[test]
pub fn sendto_addr() {
    // script_demo();
    // Client::new(sockpath)
    let client =
        clightningrpc::LightningRPC::new("./../../.docker/volumes/lightningd_data/lightning-rpc");
    // let client2 =
    //     clightningrpc::LightningRPC::new("./../../.docker/volumes/lightningd2_data/lightning-rpc");

    dbg!(client.getinfo().unwrap());
    // let id = client.getinfo().unwrap().id;
    // let result = client.connect(&id, Some("10.5.0.5:19846")).unwrap();
    // client.invoice(msatoshi, label, description, expiry)
    // client.invoice(msatoshi, label, description, expiry)
    // dbg!(result);
    let fund_addr = client.newaddr(None).unwrap();
    dbg!(fund_addr);
}

// #[tokio::test]
pub async fn connect_lnd_and_lightingd() {
    let mut lnd = connect_lightning(
        "10.5.0.6".to_string(),
        10006,
        "/home/ash/.docker/volumes/lnd_data/tls.cert".to_owned(),
        "/home/ash/.docker/volumes/lnd_data/admin.macaroon".to_owned(),
    )
    .await
    .expect("failed to connect");

    let lightningd =
        clightningrpc::LightningRPC::new("/home/ash/.docker/volumes/lightningd_data/lightning-rpc");
        dbg!(lnd.get_info(GetInfoRequest{}).await.unwrap());
    let getinfo = lightningd.getinfo().unwrap();
    
    let node_pubkey = getinfo.id;
    let host_address = match getinfo.binding[1].clone() {
        clightningrpc::responses::NetworkAddress::Ipv4 { address, port } => {
            address.to_string() + ":" + &port.to_string()
        }
        clightningrpc::responses::NetworkAddress::Ipv6 { address, port } => {
            address.to_string() + ":" + &port.to_string()
        }
        clightningrpc::responses::NetworkAddress::Torv2 { address, port } => {
            address.to_string() + ":" + &port.to_string()
        }
        clightningrpc::responses::NetworkAddress::Torv3 { address, port } => {
            address.to_string() + ":" + &port.to_string()
        }
    };
    println!("connecting to address{} ", host_address);
    let lightning_address = LightningAddress {
        pubkey: node_pubkey.clone(),
        host: "10.5.0.2:19846".to_string(),
    };

    println!("Mine to address bcrt1prnpxwf9tpjm4jll4ts72s2xscq66qxep6w9hf6sqnvwe9t4gvqasklfhyj");

    let client = RegtestCall::init(
        &vec!["bcrt1prnpxwf9tpjm4jll4ts72s2xscq66qxep6w9hf6sqnvwe9t4gvqasklfhyj"],
        "my_wallet",
        110,
    );


    let connect = ConnectPeerRequest {
        addr: Some(lightning_address),
        perm: true,
        timeout: 30,
    };

    let connect_peer = lnd.connect_peer(connect).await.unwrap();

    println!("connect peer {:#?}", connect_peer);
    dbg!(connect_peer);

    let new_address = lnd
        .new_address(NewAddressRequest {
            account: "".to_string(),
            r#type: 4,
        })
        .await
        .unwrap()
        .get_ref()
        .address
        .clone();

    println!(
        "clighting and lnd channel multi-sig address \n {}",
        new_address.clone()
    );

    P2TR::new(Some(SEED), &client).send(single_output_with_value(new_address));

    let open_channel_req = OpenChannelRequest {
        sat_per_vbyte: 30,
        node_pubkey: hex::decode(node_pubkey).unwrap(),
        local_funding_amount: 1000000,
        node_pubkey_string: "".to_owned(),
        push_sat: 750000,
        target_conf: 0,
        sat_per_byte: 0,
        private: false,
        min_htlc_msat: 100,
        remote_csv_delay: 600,
        min_confs: 0,
        spend_unconfirmed: true,
        close_address: "bcrt1prnpxwf9tpjm4jll4ts72s2xscq66qxep6w9hf6sqnvwe9t4gvqasklfhyj"
            .to_owned(),
        funding_shim: None,
        remote_max_value_in_flight_msat: 880000000,
        remote_max_htlcs: 10,
        max_local_csv: 900,
        commitment_type: 0,
        zero_conf: false,
        scid_alias: false,
        base_fee: 700,
        fee_rate: 0,
        use_base_fee: false,
        use_fee_rate: false,
        remote_chan_reserve_sat: 20000,
    };
    

    client.mine(50);

    let open_channel_response = lnd.open_channel(open_channel_req).await.unwrap();

    println!("open channel request {:#?}", open_channel_response);

    let list_peer_request = ListPeersRequest { latest_error: true };

    println!(
        "list peers {:#?}",
        lnd.list_peers(list_peer_request).await.unwrap()
    );
}
