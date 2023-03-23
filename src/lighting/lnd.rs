use std::collections::HashMap;

use bitcoin_hashes::Hash;
use bitcoincore_rpc::jsonrpc::client;
use hex::FromHex;
use tonic::{async_trait, codegen::InterceptedService, Response, Streaming};
use tower::util::Optional;
use traproot_bdk::{
    connect_lightning, connect_router,
    lnrpc::{
        lightning_client::LightningClient, AddInvoiceResponse, Channel, ConnectPeerRequest,
        ConnectPeerResponse, GetInfoRequest, GetInfoResponse, Invoice, LightningAddress,
        ListChannelsRequest, ListChannelsResponse, ListInvoiceRequest, ListInvoiceResponse,
        ListPeersRequest, ListPeersResponse, NewAddressRequest, OpenChannelRequest,
        OpenStatusUpdate, Peer, SendRequest, SendResponse,
    },
    routerrpc::SendPaymentRequest,
    MacaroonInterceptor, MyChannel,
};

use super::{AddrType, RLightningCli, WLightningCli};

pub struct Lnd {
    client: LightningClient<InterceptedService<MyChannel, MacaroonInterceptor>>,
    peer: Vec<u8>,
}

impl Lnd {
    pub async fn new() -> Self {
      
        return Lnd {
            client: connect_lightning(
                "10.5.0.6".to_string(),
                10006,
                "/home/ash/.docker/volumes/lnd_data/tls.cert".to_owned(),
                "/home/ash/.docker/volumes/lnd_data/admin.macaroon".to_owned(),
            )
            .await
            .expect("failed to connect"),
            peer: vec![],
        };
    }
}

#[async_trait]
impl
    WLightningCli<
        Response<ConnectPeerResponse>,
        Response<Streaming<OpenStatusUpdate>>,
        Response<AddInvoiceResponse>,
    > for Lnd
{
    async fn connect(&mut self, id: String, host: String) -> Response<ConnectPeerResponse> {
        let lightning_address = LightningAddress { pubkey: id, host };
        let connect_req = ConnectPeerRequest {
            addr: Some(lightning_address),
            perm: true,
            timeout: 30,
        };
        return self.client.connect_peer(connect_req).await.unwrap();
    }

    async fn new_address(&mut self, address_type: AddrType) -> String {
        let address = match address_type {
            AddrType::Bech32 => 0,
            AddrType::P2SH => 1,
            AddrType::TR => 4,
        };

        return self
            .client
            .new_address(NewAddressRequest {
                account: "".to_string(),
                r#type: address,
            })
            .await
            .unwrap()
            .get_ref()
            .address
            .clone();
    }

    async fn open_channel(
        &mut self,
        id: String,
        amt: Option<u64>,
    ) -> Response<Streaming<OpenStatusUpdate>> {
        // self.client.open_channel(request)
        let open_channel_req = OpenChannelRequest {
            sat_per_vbyte: 30,
            node_pubkey: hex::decode(id).unwrap(),
            local_funding_amount: amt.unwrap_or(0).try_into().unwrap(),
            node_pubkey_string: "".to_owned(),
            push_sat: 0,
            target_conf: 0,
            sat_per_byte: 0,
            private: false,
            min_htlc_msat: 0,
            remote_csv_delay: 0,
            min_confs: 0,
            spend_unconfirmed: true,
            close_address: "bcrt1prnpxwf9tpjm4jll4ts72s2xscq66qxep6w9hf6sqnvwe9t4gvqasklfhyj"
                .to_owned(),
            funding_shim: None,
            remote_max_value_in_flight_msat: 0,
            remote_max_htlcs: 0,
            max_local_csv: 0,
            commitment_type: 0,
            zero_conf: false,
            scid_alias: false,
            base_fee: 700,
            fee_rate: 0,
            use_base_fee: false,
            use_fee_rate: false,
            remote_chan_reserve_sat: 0,
        };
        return self.client.open_channel(open_channel_req).await.unwrap();
    }

    async fn create_invoice(
        &mut self,
        value: u64,
        label: &str,
        description: &str,
        expiry: Option<u64>,
    ) -> Response<AddInvoiceResponse> {
        // let description_hash =
        //     bitcoin_hashes::sha256::Hash::hash(&Vec::from_hex(description).unwrap()).to_vec();
        let invoice = Self::new_invoice(
            label.to_string(),
            vec![],
            value.try_into().unwrap(),
            vec![],
            expiry.unwrap_or(6555).try_into().unwrap(),
            "bcrt1qzvsdwjay5x69088n27h0qgu0tm4u6gwqgxna9d".to_string(),
            6555,
            false,
            false,
        );

        let invoice = self.client.add_invoice(invoice).await.unwrap();

        println!("invoice response {:#?}", invoice);
        return invoice;
    }
}
impl Lnd {
    pub fn new_invoice(
        memo: String,
        r_preimage: Vec<u8>,
        value: i64,
        description_hash: Vec<u8>,
        expiry: i64,
        fallback_addr: String,
        cltv_expiry: u64,
        private: bool,
        is_amp: bool,
    ) -> Invoice {
        return Invoice {
            memo,
            r_preimage,
            r_hash: vec![],
            value,
            value_msat: 0,
            settled: false,
            creation_date: 0,
            settle_date: 0,
            payment_request: "".to_string(),
            description_hash,
            expiry,
            fallback_addr,
            cltv_expiry,
            route_hints: vec![],
            private,
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
            is_amp,
            amp_invoice_state: HashMap::new(),
        };
    }
}

#[async_trait]
impl
    RLightningCli<
        Response<GetInfoResponse>,
        Response<ListPeersResponse>,
        Response<ListChannelsResponse>,
        Response<ListInvoiceResponse>,
    > for Lnd
{
    async fn get_info(&mut self) -> Response<GetInfoResponse> {
        return self.client.get_info(GetInfoRequest {}).await.unwrap();
    }

    async fn list_peers(&mut self) -> Response<ListPeersResponse> {
        return self
            .client
            .list_peers(ListPeersRequest { latest_error: true })
            .await
            .unwrap();
    }

    async fn list_channels(&mut self) -> Response<ListChannelsResponse> {
        return self
            .client
            .list_channels(ListChannelsRequest {
                active_only: true,
                inactive_only: false,
                public_only: false,
                private_only: false,
                peer: self.peer.clone(),
            })
            .await
            .unwrap();
    }
    async fn list_invoices(&mut self) -> Response<ListInvoiceResponse> {
        return self
            .client
            .list_invoices(ListInvoiceRequest {
                pending_only: false,
                index_offset: 0,
                num_max_invoices: 100,
                reversed: false,
                creation_date_end: 0,
                creation_date_start: 0,
            })
            .await
            .unwrap();
    }
}
impl Lnd {
    pub async fn send_payment(
        &mut self,
        bolt11:String
    ) -> SendResponse {

        let send_req = SendRequest {
            allow_self_payment: true,
            amt:0,
            amt_msat: 0,
            cltv_limit: 0,
            dest: vec![],
            dest_custom_records: HashMap::new(),
            dest_features: vec![],
            dest_string: "".to_owned(),
            fee_limit: None,
            final_cltv_delta: 1000,
            last_hop_pubkey: vec![],
            outgoing_chan_id: 0,
            payment_addr: vec![],
            payment_hash:vec![],
            payment_hash_string: "".to_string(),
            payment_request: bolt11.clone(),
        };

        return self
            .client
            .send_payment_sync(send_req)
            .await
            .unwrap()
            .get_ref()
            .clone();
    }
}
