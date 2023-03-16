use std::{
    collections::HashMap,
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use bitcoin_hashes::{hex::FromHex, Hash};
use clightningrpc::{responses::{Connect, GetInfo, ListChannels, ListPeers, ListInvoice, ListInvoices}, responses::FundChannel, LightningRPC, };
use tokio::task;
use tonic::{async_trait, codegen::InterceptedService};
use traproot_bdk::{
    connect_lightning,
    lnrpc::{
        lightning_client::LightningClient, ConnectPeerRequest, Invoice, LightningAddress,
        ListInvoiceRequest, ListPeersRequest, NewAddressRequest, OpenChannelRequest,
    },
    MacaroonInterceptor, MyChannel,
};

use crate::{
    bitcoin_wallet::{constants::SEED, input_data::regtest_call::RegtestCall},
    simple_wallet::{p2tr_key::P2TR, single_output_with_value, Wallet},
};

use super::{WLightningCli, RLightningCli};

pub async fn get_lnd_client() -> LightningClient<InterceptedService<MyChannel, MacaroonInterceptor>>
{
    return connect_lightning(
        "10.5.0.6".to_string(),
        10006,
        "/home/ash/.docker/volumes/lnd_data/tls.cert".to_owned(),
        "/home/ash/.docker/volumes/lnd_data/admin.macaroon".to_owned(),
    )
    .await
    .expect("failed to connect");
}

pub fn get_lightind_client() -> LightningRPC {
    return clightningrpc::LightningRPC::new(
        "/home/ash/.docker/volumes/lightningd_data/lightning-rpc",
    );
}

#[tokio::test]
pub async fn connect_lnd_and_lightingd() {
    let mut lnd = get_lnd_client().await;

    let lightningd = get_lightind_client();

    let getinfo = lightningd.getinfo().unwrap();

    let node_pubkey = getinfo.id;

    let lightning_address = LightningAddress {
        pubkey: node_pubkey.clone(),
        host: "10.5.0.5:19846".to_string(),
    };

    let client = RegtestCall::init(
        &vec!["bcrt1prnpxwf9tpjm4jll4ts72s2xscq66qxep6w9hf6sqnvwe9t4gvqasklfhyj"],
        "my_wallet",
        110,
    );

    println!("Mine to address bcrt1prnpxwf9tpjm4jll4ts72s2xscq66qxep6w9hf6sqnvwe9t4gvqasklfhyj");
    let connect = ConnectPeerRequest {
        addr: Some(lightning_address),
        perm: true,
        timeout: 30,
    };

    thread::sleep(Duration::from_secs(2));

    let connect_peer = lnd.connect_peer(connect).await.unwrap();

    println!("connect peer {:#?}", connect_peer);

    thread::sleep(Duration::from_secs(2));

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

    client.mine(50);

    thread::sleep(Duration::from_secs(2));

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

    let open_channel_response = lnd.open_channel(open_channel_req).await.unwrap();

    thread::sleep(Duration::from_secs(1));

    println!("open channel request {:#?}", open_channel_response);

    let list_peer_request = ListPeersRequest { latest_error: true };

    println!(
        "list peers {:#?}",
        lnd.list_peers(list_peer_request).await.unwrap()
    );
}

#[tokio::test]
pub async fn create_invoice() {
    let image_str = "107661134f21fc7c02223d50ab9eb3600bc3ffc3712423a1e47bb1f9a9dbf55f";
    let preimage = Vec::from_hex(image_str).unwrap();
    let r_hash = bitcoin_hashes::sha256::Hash::hash(&preimage).to_vec();
    let mut lnd = get_lnd_client().await;
    let invoice = Invoice {
        memo: "The payment description title".to_string(),
        r_preimage: preimage,
        r_hash,
        value: 1000,
        value_msat: 0,
        settled: false,
        creation_date: 0,
        settle_date: 0,
        payment_request: "".to_string(),
        description_hash: vec![],
        expiry: 7200,
        fallback_addr: "bcrt1qzvsdwjay5x69088n27h0qgu0tm4u6gwqgxna9d".to_string(),
        cltv_expiry: 100,
        route_hints: vec![],
        private: false,
        add_index: 0,
        settle_index: 0,
        amt_paid: 0,
        amt_paid_sat: 0,
        amt_paid_msat: 0,
        state: 0,
        htlcs: vec![],
        features: HashMap::new(),
        is_keysend: false,
        payment_addr: vec![],
        is_amp: false,
        amp_invoice_state: HashMap::new(),
    };
    let invoice = lnd.add_invoice(invoice).await.unwrap();
    println!("invoice response {:#?}", invoice);
}

#[tokio::test]
pub async fn list_invoices() {
    let mut lnd = get_lnd_client().await;
    let response = lnd
        .list_invoices(ListInvoiceRequest {
            pending_only: false,
            index_offset: 0,
            num_max_invoices: 100,
            reversed: false,
            creation_date_start: 0,
            creation_date_end: 0,
        })
        .await
        .unwrap();
    println!("invoices {:#?}", response);
}

pub struct Lightingd {
    client: LightningRPC,
}

#[async_trait]
impl WLightningCli<Connect, FundChannel, clightningrpc::responses::Invoice> for Lightingd {
    async fn connect(&mut self, id: String, host: String) -> Connect {
        return self.client.connect(&id, Some(&host)).unwrap();
    }

    async fn new_address(&mut self) -> String {
        return self.client.newaddr(None).unwrap().address.unwrap();
    }

    async fn open_channel(&mut self, id: String, amt: Option<u64>) -> FundChannel {
        let amount = amt
            .map(|i| clightningrpc::requests::AmountOrAll::Amount(i))
            .unwrap_or(clightningrpc::requests::AmountOrAll::All);
        return self.client.fundchannel(&id, amount, None).unwrap();
    }

    async fn create_invoice(
        &mut self,
        msatoshi: u64,
        label: &str,
        description: &str,
        expiry: Option<u64>,
    ) -> clightningrpc::responses::Invoice {
        return self
            .client
            .invoice(msatoshi, label, description, expiry)
            .unwrap();
    }
}
#[async_trait]
impl RLightningCli<GetInfo,ListPeers,ListChannels, ListInvoices> for Lightingd {
    async fn get_info(&mut self)->GetInfo{
        return self.client.getinfo().unwrap();
    }

    async fn list_peers(&mut self)->ListPeers{
        return self.client.listpeers(None , None).unwrap();
    }

    async fn list_channels(&mut self)->ListChannels{
        return self.client.listchannels(None).unwrap();
    }

    async fn list_invoices(&mut self)->ListInvoices{
        return self.client.listinvoices(None).unwrap();
     }
}
 