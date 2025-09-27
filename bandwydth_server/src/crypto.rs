/// Generate self-signed certificate for QUIC server
pub fn generate_self_signed_cert() -> Result<(Vec<u8>, Vec<u8>), Box<dyn std::error::Error + Send + Sync>> {
    let subject_alt_names = vec!["localhost".to_string()];
    let cert = rcgen::generate_simple_self_signed(subject_alt_names)?;
    let cert_der = cert.cert.der().to_vec();
    let private_key_der = cert.signing_key.serialize_der();
    Ok((cert_der, private_key_der))
}
