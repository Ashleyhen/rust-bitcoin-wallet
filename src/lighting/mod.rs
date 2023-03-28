use tonic::async_trait;

pub mod clighting;
pub mod lighting_demo;
pub mod lnd;

#[async_trait]
pub trait LNCommon<I> {
    async fn get_info(&mut self) -> I;
}

#[async_trait]
pub trait LNPeers<R, P> {
    async fn connect(&mut self, id: String, host: String) -> R;
    async fn list_peers(&mut self) -> P;
}

#[async_trait]
pub trait LNChannel<F, C> {
    async fn new_address(&mut self, address_type: AddrType) -> String;
    async fn open_channel(&mut self, id: String, amt: Option<u64>) -> F;
    async fn list_channels(&mut self) -> C;
}

#[async_trait]
pub trait LNInvoice<I, L, S> {
    async fn create_invoice(
        &mut self,
        value: u64,
        label: &str,
        description: &str,
        expiry: Option<u64>,
    ) -> I;
    async fn list_invoices(&mut self) -> L;
    async fn send_payment<'a>(&mut self, bolt11: &'a String) -> S;
}

pub enum AddrType {
    Bech32,
    TR,
    P2SH,
}
