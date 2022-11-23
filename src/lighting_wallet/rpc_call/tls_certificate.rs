use bitcoin_hashes::hex::{ToHex};
use futures::FutureExt;
use hyper::body::Bytes;
use reqwest::{Client};
use std::fs::File;
use std::io::{Read};

pub struct LnClient {
    client: Client
 }

const URL:&str ="https://127.0.0.1:8081";

impl LnClient {
    /// Reads the certificate in the pem format (other formats might work too, not tested)
    pub fn new()->LnClient {

        let mut cert_file = vec![];
        File::open("/home/ash/.polar/networks/1/volumes/lnd/alice/tls.cert").unwrap().read_to_end(&mut cert_file).unwrap();
        
        let cert =reqwest::Certificate::from_pem(&cert_file).unwrap();

        let client =reqwest::ClientBuilder::new()
        .add_root_certificate(cert)
        .build().unwrap();

        return LnClient{client};
        
    }
    pub async fn get<I>(&self, version:u8, method:&str, consumer:I)
        where I : Fn(Bytes)
    {
        let mut macroon_file = vec![];
        let v="/v".to_owned()+&version.to_string()+"/";
        File::open("/home/ash/.polar/networks/1/volumes/lnd/alice/data/chain/bitcoin/regtest/admin.macaroon").unwrap().read_to_end(&mut macroon_file).unwrap();
        let req =self.client.get(URL.to_owned()+&v+method).header("Grpc-Metadata-macaroon",macroon_file.to_hex());
        let future =req.send().boxed().await; 
        let response = future.unwrap();
        consumer(response.bytes().await.unwrap());
        
    }
}
