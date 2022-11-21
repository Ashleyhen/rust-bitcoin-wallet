use bitcoin::consensus::ReadExt;
use futures::FutureExt;
use httpbis::{ClientTlsOption, HeaderValue};
use hyper::client::HttpConnector;
use hyper::{Client, HeaderMap};
use hyper_rustls::HttpsConnector;
use rustls::{
    Certificate, ClientConfig, ClientConnection, ConfigBuilder, RootCertStore, ServerConfig,
    ServerName,
};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Result, Read};
use std::process::Command;
use std::str::FromStr;
use std::sync::Arc;
use std::{net::IpAddr, path::Path};

/// Represents bytes of the certificate
/// could be used to create `grpc::Client`
pub struct TLSCertificate {
    raw: Certificate,
}

impl TLSCertificate {
    /// Reads the certificate in the pem format (other formats might work too, not tested)
    /// from a file at the path
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        use std::io::{Error, ErrorKind};

        let output = Command::new("openssl")
            .args(&["x509", "-outform", "der", "-in"])
            .arg(path.as_ref())
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(Error::new(ErrorKind::InvalidInput, error.as_ref()));
        }

        Ok(TLSCertificate {
            raw: Certificate(output.stdout),
        })
    }

    /// Creates the tls using this certificate
    pub fn into_tls(self, host: &str) -> hyper::Client<HttpsConnector<HttpConnector>> {
        let mut root = RootCertStore::empty();
        root.add(&self.raw).unwrap();

        let config = ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root)
            .with_no_client_auth();
        // "https://127.0.0.1:8081"
        let rt = tokio::runtime::Runtime::new().unwrap();

        let url = (host).parse().unwrap();

        let https = hyper_rustls::HttpsConnectorBuilder::new()
            .with_tls_config(config)
            .https_only()
            .enable_http1()
            .build();

        let client: Client<_, hyper::Body> = Client::builder().build(https);
        let res = rt.block_on(client.get(url)).unwrap();
        dbg!(res);
        return client;
    }
    pub fn temp(self) {
        let mut cert_file = vec![];
        let mut macroon_file = "".to_string();
        let rt = tokio::runtime::Runtime::new().unwrap();
        File::open("/home/ash/.polar/networks/1/volumes/lnd/alice/tls.cert").unwrap().read_to_end(&mut cert_file).unwrap();
        File::open("/home/ash/.polar/networks/1/volumes/lnd/alice/tls.cert").unwrap().read_to_string(&mut macroon_file).unwrap();
    
        let cert =reqwest::Certificate::from_pem(&cert_file).unwrap();

        let mut headers =HeaderMap::new();
        // HeaderValue::

        headers.append("MACAROON_HEADER", macroon_file.parse().unwrap());

        

        let client =reqwest::ClientBuilder::new()
        .add_root_certificate(cert)
        .default_headers(headers)
        .build();
        
        // client.unwrap().get(url)
        
        
        
        
        let future = client.unwrap().get("https://127.0.0.1:8081/v1/fees").build();
        let resp = future.unwrap();

        println!("{:#?}", resp);
    }
}
