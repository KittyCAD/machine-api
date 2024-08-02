/// A no-op implementation of `ServerCertVerifier` that accepts any certificate.
#[derive(Debug)]
pub(crate) struct NoAuth {}

impl NoAuth {
    /// Creates a new `NoAuth` instance.
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl rustls::client::danger::ServerCertVerifier for NoAuth {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::RSA_PKCS1_SHA1,
            rustls::SignatureScheme::ECDSA_SHA1_Legacy,
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::RSA_PKCS1_SHA384,
            rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
            rustls::SignatureScheme::RSA_PKCS1_SHA512,
            rustls::SignatureScheme::ECDSA_NISTP521_SHA512,
            rustls::SignatureScheme::RSA_PSS_SHA256,
            rustls::SignatureScheme::RSA_PSS_SHA384,
            rustls::SignatureScheme::RSA_PSS_SHA512,
            rustls::SignatureScheme::ED25519,
            rustls::SignatureScheme::ED448,
        ]
    }
}

///// A no-op implementation of `ServerCertVerifier` that accepts any certificate.
//#[derive(Debug)]
//pub(crate) struct SuppaNoAuth {}
//
//impl SuppaNoAuth {
//    /// Creates a new `NoAuth` instance.
//    pub(crate) fn new() -> Self {
//        Self {}
//    }
//}
//
//impl suppaftp::rustls::client::ServerCertVerifier for SuppaNoAuth {
//    fn verify_server_cert(
//        &self,
//        _end_entity: &suppaftp::rustls::Certificate,
//        _intermediates: &[suppaftp::rustls::Certificate],
//        _server_name: &suppaftp::rustls::client::ServerName,
//        _scts: &mut dyn Iterator<Item = &[u8]>,
//        _ocsp_response: &[u8],
//        _now: std::time::SystemTime,
//    ) -> Result<suppaftp::rustls::client::ServerCertVerified, suppaftp::rustls::Error> {
//        Ok(suppaftp::rustls::client::ServerCertVerified::assertion())
//    }
//
//    fn verify_tls12_signature(
//        &self,
//        _message: &[u8],
//        _cert: &suppaftp::rustls::Certificate,
//        _dss: &suppaftp::rustls::DigitallySignedStruct,
//    ) -> Result<suppaftp::rustls::client::HandshakeSignatureValid, suppaftp::rustls::Error> {
//        Ok(suppaftp::rustls::client::HandshakeSignatureValid::assertion())
//    }
//
//    fn verify_tls13_signature(
//        &self,
//        _message: &[u8],
//        _cert: &suppaftp::rustls::Certificate,
//        _dss: &suppaftp::rustls::DigitallySignedStruct,
//    ) -> Result<suppaftp::rustls::client::HandshakeSignatureValid, suppaftp::rustls::Error> {
//        Ok(suppaftp::rustls::client::HandshakeSignatureValid::assertion())
//    }
//
//    fn supported_verify_schemes(&self) -> Vec<suppaftp::rustls::SignatureScheme> {
//        vec![
//            suppaftp::rustls::SignatureScheme::RSA_PKCS1_SHA1,
//            suppaftp::rustls::SignatureScheme::ECDSA_SHA1_Legacy,
//            suppaftp::rustls::SignatureScheme::RSA_PKCS1_SHA256,
//            suppaftp::rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
//            suppaftp::rustls::SignatureScheme::RSA_PKCS1_SHA384,
//            suppaftp::rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
//            suppaftp::rustls::SignatureScheme::RSA_PKCS1_SHA512,
//            suppaftp::rustls::SignatureScheme::ECDSA_NISTP521_SHA512,
//            suppaftp::rustls::SignatureScheme::RSA_PSS_SHA256,
//            suppaftp::rustls::SignatureScheme::RSA_PSS_SHA384,
//            suppaftp::rustls::SignatureScheme::RSA_PSS_SHA512,
//            suppaftp::rustls::SignatureScheme::ED25519,
//            suppaftp::rustls::SignatureScheme::ED448,
//        ]
//    }
//}
