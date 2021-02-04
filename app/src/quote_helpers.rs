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

    let p_sig_data: *const sgx_ql_ecdsa_sig_data_t = quote_signature_data_vec.as_ptr() as _;
    let sig_data = unsafe { *p_sig_data };

    // println!("{:?}", PublicKey::from_sec1_bytes(&sig_data.attest_pub_key));

    let enc_point = EncodedPoint::from_affine_coordinates(
        FieldBytes::from_slice(&sig_data.attest_pub_key[0..32]),
        FieldBytes::from_slice(&sig_data.attest_pub_key[32..]),
        false,
    );

    let pub_key: PublicKey = PublicKey::from_encoded_point(&enc_point).unwrap();

    let pub_key_str = pub_key.to_public_key_pem();

    println!("{:?}", pub_key_str);

    let mut hasher = Sha256::new();
    hasher.update(pub_key.to_public_key_der().spki().subject_public_key);
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

    // println!("{:?}", BASE64URL.encode(&quote));
}
