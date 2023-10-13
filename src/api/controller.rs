use bitcoin_hashes::hex::FromHex;
use rocket::{post, futures::{FutureExt, io::Cursor}, FromForm, form::Form, serde::json::Json, Response, http::{ContentType, Status, Header}, response::Responder};
use serde::{Deserialize, Serialize};
use traproot_bdk::lnrpc::Payment;

use crate::{lighting::lnd::Lnd, service::handler::{LnSendPaymentRequest, send_invoice_to}};

pub async fn send_payment(dest:String, id:i64)-> tonic::Response<tonic::Streaming<Payment>>{
    println!("qr-code id: {}",id);
    let mut lnd_client = Lnd::new().await;
    let result =lnd_client.send_amp_payment(Vec::from_hex(&dest).unwrap(), 1000).await;
    dbg!(&result);
        return result;
}



#[derive(Responder,Clone,Debug)]
#[response(content_type = "application/x-person")]
pub struct PaymentRequest<'a> {
    body:String,
    header:Header<'a>
}

#[post("/lnurl",format = "json", data = "<form>")]
 pub async fn post_handler(form: Json<LnSendPaymentRequest>) -> PaymentRequest<'static>{

    let ln_spend_payment_request=form.into_inner();

        send_invoice_to(&ln_spend_payment_request).await;

     let payment_request=PaymentRequest{
        header: Header::new("X-Custom-Header","custom value"), 
        body:ln_spend_payment_request.bolt11 
    };

    return payment_request;
}

