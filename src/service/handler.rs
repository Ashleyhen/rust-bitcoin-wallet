use lightning_invoice::Invoice;
use serde::{Deserialize, Serialize};

use crate::lighting::{lnd::Lnd, LNInvoice};

#[derive(Serialize, Deserialize,rocket::FromForm,Clone,Debug)]
pub struct LnSendPaymentRequest {
    pub bolt11: String,
    pub id:i64
}

pub async fn send_invoice_to(ln_send_payment:&LnSendPaymentRequest){
    let mut lnd_client = Lnd::new().await;

    let millisat=ln_send_payment.bolt11.parse::<Invoice>().unwrap().amount_milli_satoshis().unwrap();
    if(millisat>=200000000){
        panic!("msats: {} too many stats taken",millisat);

    }
    dbg!(lnd_client.send_payment(&ln_send_payment.bolt11).await);
    
}
#[tokio::test]
pub async fn create_invoice(){

    let invoice=dbg!(Lnd::new_1().await.create_invoice(20000, "pay me", "give me money", None).await.into_inner());
    
    // let mut lnd=Lnd::new().await;
    // lnd.send_payment(&invoice.payment_request).await;

}