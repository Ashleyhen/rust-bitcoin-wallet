use std::{collections::HashMap, fmt::Debug, thread, time::Duration};

use bitcoin::secp256k1::Scalar;
use bitcoin_hashes::{hex::ToHex, Hash};
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
    bitcoin_wallet::{
        constants::SEED,
        input_data::{regtest_call::RegtestCall, RpcCall},
    },
    lighting::{AddrType, LNChannel, LNCommon, LNInvoice, LNPeers},
    simple_wallet::{
        p2tr_key::P2TR, p2wpkh::P2WPKH, single_output, single_output_with_value, Wallet,
    },
};

use super::{clighting::Lightingd, lnd::Lnd};

pub async fn connect_open_channel<R, P, F, C, L>(
    mut ln_client: L,
    client: RegtestCall,
    host: &str,
    pub_id: &str,
) where
    R: Sized + Debug,
    P: Sized + Debug,
    F: Debug,
    L: LNPeers<R, P> + LNChannel<F, C>,
{
    thread::sleep(Duration::from_secs(2));

    let lnd_to_lightind = ln_client.connect(pub_id.to_owned(), host.to_owned()).await;

    println!("connect peer {:#?}", lnd_to_lightind);
    dbg!(ln_client.list_peers().as_mut().await);
    client.mine(20);
    let new_address = ln_client.new_address(AddrType::TR).await;
    P2TR::new(Some(SEED), &client).send(single_output_with_value(new_address.clone()));

    client.mine(100);

    println!(
        "clighting and lnd channel multi-sig address \n {}",
        new_address.clone()
    );

    thread::sleep(Duration::from_secs(3));

    let open_channel_response = ln_client
        .open_channel(pub_id.to_owned(), Some(10000000))
        .await;

    println!("open channel request {:#?}", open_channel_response);

    client.mine(20);

    println!("Testing layer 1 pay to tap root with key signature");
}


#[tokio::test]
pub async fn open_channel_request() {
    let lnd_client = Lnd::new().await;
    let mut lnd_client_1 = Lnd::new_1().await;

    let client = RegtestCall::init(
        &vec!["bcrt1prnpxwf9tpjm4jll4ts72s2xscq66qxep6w9hf6sqnvwe9t4gvqasklfhyj"],
        "my_wallet",
        110,
    );

    let id = lnd_client_1
        .get_info()
        .await
        .get_ref()
        .identity_pubkey
        .clone();

    connect_open_channel(lnd_client, client, "10.5.0.7:9730", &id).await
}

#[tokio::test]
pub async fn clighting_sends_open_channel_request() {
    let mut lnd = Lnd::new().await;
    let mut lightingd = Lightingd::new().await;
    let get_info = lnd.get_info().await;

    let str_address = "bcrt1qzvsdwjay5x69088n27h0qgu0tm4u6gwqgxna9d";

    println!("Testing layer 1 pay to witness public key signature");
    let client = RegtestCall::init(&vec![str_address], "my_wallet", 110);

    thread::sleep(Duration::from_secs(2));

    let lnd_to_lightind = lightingd
        .connect(
            get_info.get_ref().clone().identity_pubkey,
            "10.5.0.6:10006".to_string(),
        )
        .await;

    thread::sleep(Duration::from_secs(2));
    println!("connect peer {:#?}", lnd_to_lightind);

    client.mine(20);
    let new_address = lightingd.new_address(AddrType::Bech32).await;

    P2WPKH::new(Some(SEED), &client).send(single_output_with_value(new_address.clone()));
    client.mine(100);

    println!(
        "clighting and lnd channel multi-sig address \n {}",
        new_address.clone()
    );

    thread::sleep(Duration::from_secs(3));
    let open_channel_response = lightingd
        .open_channel(lnd.get_info().await.get_ref().identity_pubkey.clone(), None)
        .await;

    client.mine(20);
    println!("open channel request {:#?}", open_channel_response);
}



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
    let new_address = lnd.new_address(AddrType::TR).await;
    P2TR::new(Some(SEED), &client).send(single_output_with_value(new_address.clone()));
    client.mine(100);

    println!(
        "clighting and lnd channel multi-sig address \n {}",
        new_address.clone()
    );

    thread::sleep(Duration::from_secs(3));

    let open_channel_response = lnd
        .open_channel(lightingd.get_info().await.id, Some(10000000))
        .await;

    println!("open channel request {:#?}", open_channel_response);

    client.mine(20);

    thread::sleep(Duration::from_secs(2));

    let invoice = lightingd
        .create_invoice(
            1000,
            "invoice from lightingd",
            "payment description!",
            Some(7200),
        )
        .await;

    println!("print out invoice {:#?}", invoice);

    let list_peer_request = lnd.list_peers().await.get_ref().clone();

    println!("list peers {:#?}", list_peer_request);

    println!("Testing layer 1 pay to tap root with key signature");
}





#[tokio::test]
pub async fn lnd_list_peer() {
    let mut lnd = Lnd::new().await;
    dbg!(lnd.list_channels().await);
}

#[tokio::test]
pub async fn lightingd_create_invoice_and_pay() {
    let mut lighting_d = Lightingd::new().await;
    let mut lnd = Lnd::new().await;

    let random_data = Scalar::random().to_be_bytes().to_hex();

    let invoice = lighting_d
        .create_invoice(1000, &random_data, "some description", None)
        .await;

    thread::sleep(Duration::from_secs(2));

    dbg!(invoice.clone());

    let payment_response = lnd.send_payment(&invoice.bolt11).await;

    println!("invoice paid: {:#?}", payment_response);
}

#[tokio::test]
pub async fn lnd_create_invoice_and_pay() {
    let mut lighting_d = Lightingd::new().await;
    let mut lnd = Lnd::new().await;

    let random_data = Scalar::random().to_be_bytes().to_hex();

    let invoice = lnd
        .create_invoice(20000, &random_data, "some description", None)
        .await;

    thread::sleep(Duration::from_secs(2));

    dbg!(invoice.get_ref().clone());

    dbg!(&lnd.get_info().await.get_ref().identity_pubkey);

    let payment_response = lighting_d
        .send_payment(&invoice.get_ref().clone().payment_request)
        .await;

    println!("invoice paid: {:#?}", payment_response);
}

#[tokio::test]
pub async fn quick_pay() {
    let mut lnd_client = Lnd::new().await;
    let mut lnd_client_1 = Lnd::new_1().await;

    let pub_key_1 =lnd_client_1.get_info().await.get_ref().identity_pubkey.clone();
     
    let result =lnd_client.send_amp_payment(Vec::from_hex(pub_key_1).unwrap(),10000).await;
    dbg!(result);


}
