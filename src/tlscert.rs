use std::collections::HashSet;

use rcgen::{
    Certificate as RcgenCert, CertificateParams, DistinguishedName, DnType, KeyPair, RcgenError,
    SanType,
};
use rustls::{
    Certificate as RustlsCert, ClientConfig as RustlsClientConfig, Error, OwnedTrustAnchor,
    PrivateKey, RootCertStore, ServerConfig as RustlsServerConfig,
};

/// DER-encoded
pub struct SingleCert {
    pub cert: Vec<u8>,
    pub key: Vec<u8>,
}

pub async fn cert_generate(alt_dnsname: &HashSet<&str>) -> Result<SingleCert, RcgenError> {
    let ca = RcgenCert::from_params(CertificateParams::from_ca_cert_pem(
        include_str!("../private/ca.crt"),
        KeyPair::from_pem(include_str!("../private/key.crt"))?,
    )?)?;

    let mut cert_params = CertificateParams::default();
    cert_params.distinguished_name = {
        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName, "snimap");
        dn
    };
    cert_params.subject_alt_names = alt_dnsname
        .iter()
        .map(|s| SanType::DnsName(s.to_string()))
        .collect();

    let server_cert = RcgenCert::from_params(cert_params)?;

    Ok(SingleCert {
        cert: server_cert.serialize_der_with_signer(&ca)?,
        key: server_cert.serialize_private_key_der(),
    })
}

pub trait DisableSni {
    fn disable_sni(self) -> Self;
}

impl DisableSni for RustlsClientConfig {
    fn disable_sni(mut self) -> Self {
        self.enable_sni = false;
        self
    }
}

pub fn rustls_client_config() -> RustlsClientConfig {
    let mut root_store = RootCertStore::empty();

    root_store.add_server_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.0.iter().map(|ta| {
        OwnedTrustAnchor::from_subject_spki_name_constraints(
            ta.subject,
            ta.spki,
            ta.name_constraints,
        )
    }));

    RustlsClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_store)
        .with_no_client_auth()
}

pub fn rustls_server_config(single_cert: SingleCert) -> Result<RustlsServerConfig, Error> {
    RustlsServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(
            vec![RustlsCert(single_cert.cert)],
            PrivateKey(single_cert.key),
        )
}
