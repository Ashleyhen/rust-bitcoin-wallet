use std::{collections::HashMap, thread, time::Duration};

use bitcoin_hashes::Hash;
use clightningrpc::LightningRPC;
use hex::FromHex;
use tonic::codegen::InterceptedService;
use traproot_bdk::{
    connect_lightning,
    lnrpc::{
        lightning_client::LightningClient, ConnectPeerRequest, Invoice, LightningAddress,
        ListPeersRequest, NewAddressRequest, OpenChannelRequest,
    },
    MacaroonInterceptor, MyChannel,
};

use crate::{
    bitcoin_wallet::{constants::SEED, input_data::regtest_call::RegtestCall},
    lighting::WLightningCli,
    simple_wallet::{p2tr_key::P2TR, single_output, single_output_with_value, Wallet},
};

use super::{clighting::Lightingd, lnd::Lnd, RLightningCli};

#[tokio::test]
pub async fn lnd_sends_open_channel_request() {
    let mut lnd = Lnd::new().await;
    let mut lightingd = Lightingd::new().await;
    let get_info = lightingd.get_info().await;

    let client = RegtestCall::init(
        &vec!["bcrt1prnpxwf9tpjm4jll4ts72s2xscq66qxep6w9hf6sqnvwe9t4gvqasklfhyj"],
        "my_wallet",
        110,
    );

    thread::sleep(Duration::from_secs(2));

    let lnd_to_lightind = lnd
        .connect(get_info.clone().id, "10.5.0.5:19846".to_string())
        .await;

    println!("connect peer {:#?}", lnd_to_lightind);

    client.mine(20);
    let new_address = lnd.new_address().await;
    P2TR::new(Some(SEED), &client).send(single_output_with_value(new_address.clone()));
    client.mine(20);

    println!(
        "clighting and lnd channel multi-sig address \n {}",
        new_address.clone()
    );

    thread::sleep(Duration::from_secs(2));
    let open_channel_response=lnd.open_channel(lightingd.get_info().await.id, Some(1000000))
        .await;

    println!("open channel request {:#?}", open_channel_response);


    client.mine(20);

    thread::sleep(Duration::from_secs(2));

    
    let invoice = lightingd
        .create_invoice(1000, "payment title", "payment description!", Some(7200))
        .await;

    println!("print out invoice {:#?}", invoice);

    let list_peer_request=lnd.list_peers().await.get_ref().clone();

    println!(
        "list peers {:#?}",
        list_peer_request
    );

    println!("Testing layer 1 pay to tap root with key signature");
}