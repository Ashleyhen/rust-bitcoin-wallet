use httpbis::ClientTlsOption;
use hyper::Client;
use hyper::client::HttpConnector;
use hyper_rustls::HttpsConnector;
use rustls::{
    Certificate, ClientConfig, ClientConnection, ConfigBuilder, RootCertStore, ServerConfig,
    ServerName,
};
use std::io::Result;
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

        // tls_api::TlsConnectorBuilderBox
        // tls_api::TlsStreamWithSocket::new(imp)

        Ok(TLSCertificate {
            raw: Certificate(output.stdout),
        })
    }

    /// Creates the tls using this certificate
    pub fn into_tls(self, host: &str) -> hyper::Client<HttpsConnector<HttpConnector>> {
        let mut root = RootCertStore::empty();
        root.add(&self.raw);

        let mut config = ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root)
            .with_no_client_auth();
        
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        let url = ("https://hyper.rs").parse().unwrap();

        let https = hyper_rustls::HttpsConnectorBuilder::new()
            .with_tls_config(config)
            .https_only()
            .enable_http1()
            .build();

        let client: Client<_, hyper::Body> = Client::builder().build(https);
        let res = rt.block_on(client.get(url)).unwrap();
        return client;
    }
}
