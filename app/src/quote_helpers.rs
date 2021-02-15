extern crate data_encoding;
extern crate itertools;
extern crate p256;
extern crate sha2;
use self::data_encoding::BASE64URL;
use self::data_encoding::HEXLOWER;
use self::itertools::*;
use self::p256::*;
use self::sha2::{Digest, Sha256};
use quote_helpers::p256::elliptic_curve::sec1::FromEncodedPoint;
use quote_helpers::p256::pkcs8::ToPublicKey;
use serde::{Deserialize, Serialize};
use serde_json::Result;
use sgx_types::*;
use std::slice;
use std::str;
use std::vec::Vec;
use std::FromStr;

#[derive(Serialize, Deserialize)]
struct QuoteRuntimeData {
    data: String,
    dataType: String,
}

#[derive(Serialize, Deserialize)]
pub struct AttestationResult {
    quote: String,
    runtimeData: QuoteRuntimeData,
}
pub fn parse_quote(quote: Vec<u8>) -> AttestationResult {
    let p_quote3: *const sgx_quote3_t = quote.as_ptr() as *const sgx_quote3_t;
    // copy heading bytes to a sgx_quote3_t type to simplify access
    let quote3: sgx_quote3_t = unsafe { *p_quote3 };
    let quote_signature_data_vec: Vec<u8> = quote[std::mem::size_of::<sgx_quote3_t>()..].into();
    assert_eq!(
        quote3.signature_data_len as usize,
        quote_signature_data_vec.len()
    );
    let auth_certification_data_offset = std::mem::size_of::<sgx_ql_ecdsa_sig_data_t>();
    let p_auth_data: *const sgx_ql_auth_data_t =
        (quote_signature_data_vec[auth_certification_data_offset..]).as_ptr() as _;
    let auth_data_header: sgx_ql_auth_data_t = unsafe { *p_auth_data };
    let auth_data_offset =
        auth_certification_data_offset + std::mem::size_of::<sgx_ql_auth_data_t>();
    let temp_cert_data_offset = auth_data_offset + auth_data_header.size as usize;
    let p_temp_cert_data: *const sgx_ql_certification_data_t =
        quote_signature_data_vec[temp_cert_data_offset..].as_ptr() as _;
    let temp_cert_data: sgx_ql_certification_data_t = unsafe { *p_temp_cert_data };
    let cert_info_offset =
        temp_cert_data_offset + std::mem::size_of::<sgx_ql_certification_data_t>();

    println!("cert-data: {:?}", temp_cert_data.certification_data);
    assert_eq!(
        quote_signature_data_vec.len(),
        cert_info_offset + temp_cert_data.size as usize
    );

    let tail_content = quote_signature_data_vec[cert_info_offset..].to_vec();
    let enc_ppid_len = 384;
    let enc_ppid: &[u8] = &tail_content[0..enc_ppid_len];
    let pce_id: &[u8] = &tail_content[enc_ppid_len..enc_ppid_len + 2];
    let cpu_svn: &[u8] = &tail_content[enc_ppid_len + 2..enc_ppid_len + 2 + 16];
    let pce_isvsvn: &[u8] = &tail_content[enc_ppid_len + 2 + 16..enc_ppid_len + 2 + 18];
    println!("EncPPID:\n{:02x}", enc_ppid.iter().format(""));
    println!("PCE_ID:\n{:02x}", pce_id.iter().format(""));
    println!("TCBr - CPUSVN:\n{:02x}", cpu_svn.iter().format(""));
    println!("TCBr - PCE_ISVSVN:\n{:02x}", pce_isvsvn.iter().format(""));
    println!("QE_ID:\n{:02x}", quote3.header.user_data.iter().format(""));

    let pub_key: PublicKey = PublicKey::from_str("-----BEGIN PUBLIC KEY-----\nMFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEypdQWcBTThlAljbSFjhLLpphhNQs\nasGkEA5ib329C9RM1w+BRqydC3PcOHWJh+A2VoEXv7XSj+MHbx/kFRmZhw==\n-----END PUBLIC KEY-----").unwrap();

    let pub_key_str = pub_key.to_public_key_pem();

    println!("{:?}", pub_key_str);

    let mut hasher = Sha256::new();
    hasher.update(&pub_key_str);
    let result = hasher.finalize();
    println!("hash: {:?}", HEXLOWER.encode(&result));

    println!(
        "app_report.body.report_data.d = {:02x}",
        quote3.report_body.report_data.d.iter().format("")
    );

    assert_eq!(
        quote3.signature_data_len as usize,
        quote_signature_data_vec.len()
    );

    AttestationResult {
        quote: BASE64URL.encode(&quote),
        runtimeData: QuoteRuntimeData {
            data: BASE64URL.encode(pub_key_str.as_bytes()),
            dataType: "Binary".to_string(),
        },
    }
}
