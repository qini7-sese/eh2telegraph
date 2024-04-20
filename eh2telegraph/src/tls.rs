use std::sync::Arc;

use rustls::{
    client::{ServerCertVerifier, WebPkiVerifier},
    Certificate, ClientConfig, RootCertStore, ServerName,
};

pub struct WhitelistVerifier<const N: usize> {
    verifier: WebPkiVerifier,
    dns_names: [&'static str; N],
}

/// Custom verifier that allow hostname difference with specified dns names.
impl<const N: usize> WhitelistVerifier<N> {
    pub fn new(dns_names: [&'static str; N]) -> Self {
        use rustls::OwnedTrustAnchor;
        let mut root_cert_store = RootCertStore::empty();
        let trust_anchors = webpki_roots::TLS_SERVER_ROOTS.0.iter().map(|trust_anchor| {
            OwnedTrustAnchor::from_subject_spki_name_constraints(
                trust_anchor.subject,
                trust_anchor.spki,
                trust_anchor.name_constraints,
            )
        });
        root_cert_store.add_server_trust_anchors(trust_anchors);
        let verifier = WebPkiVerifier::new(root_cert_store, None);
        Self {
            verifier,
            dns_names,
        }
    }
}

impl<const N: usize> From<WhitelistVerifier<N>> for ClientConfig {
    fn from(v: WhitelistVerifier<N>) -> Self {
        let mut cfg = ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(RootCertStore::empty())
            .with_no_client_auth();
        cfg.dangerous().set_certificate_verifier(Arc::new(v));
        cfg
    }
}

impl<const N: usize> ServerCertVerifier for WhitelistVerifier<N> {
    fn verify_server_cert(
        &self,
        end_entity: &Certificate,
        intermediates: &[Certificate],
        server_name: &rustls::ServerName,
        scts: &mut dyn Iterator<Item = &[u8]>,
        ocsp_response: &[u8],
        now: std::time::SystemTime,
    ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
        let original_validate_result = self.verifier.verify_server_cert(
            end_entity,
            intermediates,
            server_name,
            scts,
            ocsp_response,
            now,
        );
        if original_validate_result.is_ok() {
            return original_validate_result;
        }
        for dns_name in self.dns_names.iter() {
            if let Ok(dns_name) = ServerName::try_from(*dns_name) {
                let whitelist_validate_result = self.verifier.verify_server_cert(
                    end_entity,
                    intermediates,
                    &dns_name,
                    scts,
                    ocsp_response,
                    now,
                );
                if whitelist_validate_result.is_ok() {
                    return whitelist_validate_result;
                }
            }
        }
        original_validate_result
    }
}
