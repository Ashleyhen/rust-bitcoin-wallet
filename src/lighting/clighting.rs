use std::{
    collections::HashMap,
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use bitcoin_hashes::{hex::FromHex, Hash};
use clightningrpc::{
    responses::FundChannel,
    responses::{Connect, GetInfo, ListChannels, ListInvoice, ListInvoices, ListPeers},
    LightningRPC,
};
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

use super::{RLightningCli, WLightningCli};

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
