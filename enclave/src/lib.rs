#![crate_name = "tee_median"]
#![crate_type = "staticlib"]
#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

extern crate serde_json;
extern crate sgx_crypto_helper;
extern crate sgx_tcrypto;
#[cfg(not(target_env = "sgx"))]
extern crate sgx_tstd as std;
extern crate sgx_types;

use sgx_types::*;
use std::io::{self, Read, Write};
use std::prelude::v1::*;
use std::println;
use std::sgxfs::SgxFile;
use std::slice;
use std::string::String;
use std::vec::Vec;

use sgx_crypto_helper::rsa3072::Rsa3072KeyPair;
use sgx_crypto_helper::rsa3072::Rsa3072PubKey;
use sgx_crypto_helper::RsaKeyPair;

pub const KEYFILE: &'static str = "prov_key.bin";

#[no_mangle]
pub extern "C" fn calculate_median(slice: *mut i64, len: usize, median: &mut i64) -> sgx_status_t {
    let data: &mut [i64] = unsafe { slice::from_raw_parts_mut(slice, len) };

    data.sort_unstable();
    let mid = data.len() / 2;

    *median = data[mid];

    sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub extern "C" fn create_keypair(pubkey: &mut Rsa3072PubKey) -> sgx_status_t {
    let rsa_keypair = match Rsa3072KeyPair::new() {
        Ok(a) => a,
        _ => unreachable!(),
    };

    let rsa_key_json = serde_json::to_string(&rsa_keypair).unwrap();
    let exported_pubkey = rsa_keypair.export_pubkey().unwrap();
    *pubkey = exported_pubkey;

    match SgxFile::create(KEYFILE) {
        Ok(mut f) => match f.write_all(rsa_key_json.as_bytes()) {
            Ok(()) => {
                println!("SgxFile write key file success!");
                sgx_status_t::SGX_SUCCESS
            }
            Err(x) => {
                println!("SgxFile write key file failed! {}", x);
                sgx_status_t::SGX_ERROR_UNEXPECTED
            }
        },
        Err(x) => {
            println!("SgxFile create file {} error {}", KEYFILE, x);
            sgx_status_t::SGX_ERROR_UNEXPECTED
        }
    }
}

#[no_mangle]
pub extern "C" fn decrypt_data(ciphertext: *mut u8, len: usize) -> sgx_status_t {
    let ciphertext_slice = unsafe { slice::from_raw_parts(ciphertext, len) };
    let mut keyvec: Vec<u8> = Vec::new();

    let key_json_str = match SgxFile::open(KEYFILE) {
        Ok(mut f) => match f.read_to_end(&mut keyvec) {
            Ok(len) => {
                println!("Read {} bytes from Key file", len);
                std::str::from_utf8(&keyvec).unwrap()
            }
            Err(x) => {
                println!("Read keyfile failed {}", x);
                return sgx_status_t::SGX_ERROR_UNEXPECTED;
            }
        },
        Err(x) => {
            println!("get_sealed_pcl_key cannot open keyfile, please check if key is provisioned successfully! {}", x);
            return sgx_status_t::SGX_ERROR_UNEXPECTED;
        }
    };

    let mut plaintext = Vec::<u8>::new();

    let rsa_keypair: Rsa3072KeyPair = serde_json::from_str(&key_json_str).unwrap();

    rsa_keypair.decrypt_buffer(ciphertext_slice, &mut plaintext);

    println!("{}", String::from_utf8(plaintext).unwrap());

    sgx_status_t::SGX_SUCCESS
}
