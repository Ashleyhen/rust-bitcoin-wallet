use std::{
    collections::HashMap,
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use bitcoin_hashes::{hex::FromHex, Hash};
use bitcoincore_rpc::jsonrpc::serde_json::{self, json};
use clightningrpc::{
    requests::AmountOrAll,
    responses::FundChannel,
    responses::{Connect, GetInfo, ListChannels, ListInvoice, ListInvoices, ListPeers},
    Error, LightningRPC, Response,
};
use serde::{Deserialize, Serialize};
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

use super::{AddrType, RLightningCli, WLightningCli};

pub struct Lightingd {
    client: LightningRPC,
}
impl Lightingd {
    pub async fn new() -> Self {
        return Lightingd {
            client: clightningrpc::LightningRPC::new(
                "/home/ash/.docker/volumes/lightningd_data/lightning-rpc",
            ),
        };
    }
}

#[async_trait]
impl WLightningCli<Connect, FundChannel, clightningrpc::responses::Invoice> for Lightingd {
    async fn connect(&mut self, id: String, host: String) -> Connect {
        return self.client.connect(&id, Some(&host)).unwrap();
    }

    async fn new_address(&mut self, addr_type: AddrType) -> String {
        let address_mapping = |address: &str| self.client.newaddr(Some(address)).unwrap();

        return match addr_type {
            AddrType::Bech32 => address_mapping("bech32").bech32.unwrap(),
            AddrType::TR => address_mapping("all").address.unwrap(),
            AddrType::P2SH => address_mapping("p2sh-segwit").p2sh_segwit.unwrap(),
        };
    }

    async fn open_channel(&mut self, id: String, amt: Option<u64>) -> FundChannel {
        let amount = amt.map(|i| i.to_string()).unwrap_or("all".to_string());

        let request = OpenChannel::new(id.as_str(), &amount);
        let result: Result<Response<FundChannel>, Error> = self
            .client
            .client()
            .send_request("fundchannel", request.clone());
        return result.unwrap().result.unwrap();
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
impl RLightningCli<GetInfo, ListPeers, ListChannels, ListInvoices> for Lightingd {
    async fn get_info(&mut self) -> GetInfo {
        return self.client.getinfo().unwrap();
    }

    async fn list_peers(&mut self) -> ListPeers {
        return self.client.listpeers(None, None).unwrap();
    }

    async fn list_channels(&mut self) -> ListChannels {
        return self.client.listchannels(None).unwrap();
    }

    async fn list_invoices(&mut self) -> ListInvoices {
        return self.client.listinvoices(None).unwrap();
    }
}

#[tokio::test]
pub async fn clighting_sends_open_channel_request() {
    let mut lightingd = Lightingd::new().await;
    dbg!(lightingd.get_info().await);
}
/// 'aundchannel' command
#[derive(Debug, Clone, Deserialize, Serialize)]

// [feerate] [announce] [minconf] [utxos] [push_msat] [close_to] [request_amt] [compact_lease] [reserve]
pub struct OpenChannel<'a, 'b,  'd, 'e, 'g> {
    pub id: &'a str,
    pub amount: &'b str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feerate: Option<u64>,
    pub announce: bool,
    pub minconf: u64,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub utxos:  Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub push_msat: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub close_to: Option<&'d str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_amt: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compact_lease: Option<&'e str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reserve: Option<&'g str>,
}

impl<'a, 'b,  'd, 'e, 'g> OpenChannel<'a, 'b,  'd, 'e, 'g> {
    pub fn new(id: &'a str, amount:&'b str) -> OpenChannel <'a, 'b,  'd, 'e, 'g>{
        return OpenChannel {
            id,
            amount,
            feerate: None,
            announce: true,
            minconf: 1,
            utxos: vec![],
            push_msat: None,
            close_to: None,
            request_amt: None,
            compact_lease: None,
            reserve: None,
        };
    }
}
