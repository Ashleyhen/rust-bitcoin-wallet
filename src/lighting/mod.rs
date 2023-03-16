use tonic::async_trait;

pub mod clighting;
pub mod lnd;

#[async_trait]
pub trait WLightningCli<R, F, I> {
    async fn connect(&mut self, id: String, host: String) -> R;
    async fn new_address(&mut self) -> String;
    async fn open_channel(&mut self, id: String, amt: Option<u64>) -> F;
    async fn create_invoice(
        &mut self,
        value: u64,
        label: &str,
        description: &str,
        expiry: Option<u64>,
    ) -> I;
}

#[async_trait]
pub trait RLightningCli<G,P,C,I>{
    async fn get_info(&mut self) -> G;
    async fn list_peers(&mut self) -> P;
    async fn list_channels(&mut self) -> C;
    async fn list_invoices(&mut self) -> I;
}