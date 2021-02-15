#![crate_name = "tee_median"]
#![crate_type = "staticlib"]
#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

extern crate serde_json;
extern crate sgx_crypto_helper;
extern crate sgx_tcrypto;
// extern crate sgx_tkey_exchange;
extern crate sgx_tse;
#[cfg(not(target_env = "sgx"))]
extern crate sgx_tstd as std;
extern crate sgx_types;

use sgx_types::*;
use std::io::{Read, Write};
use std::prelude::v1::*;
use std::println;
use std::sgxfs::SgxFile;
use std::slice;
use std::vec::Vec;

use sgx_crypto_helper::rsa3072::Rsa3072KeyPair;
use sgx_crypto_helper::rsa3072::Rsa3072PubKey;
use sgx_crypto_helper::RsaKeyPair;
// use sgx_tcrypto::*;
// use sgx_tkey_exchange::*;

use sgx_tse::rsgx_create_report;

pub const KEYFILE: &'static str = "prov_key.bin";

// const G_SP_PUB_KEY: sgx_ec256_public_t = sgx_ec256_public_t {
//     gx: [
//         0x72, 0x12, 0x8a, 0x7a, 0x17, 0x52, 0x6e, 0xbf, 0x85, 0xd0, 0x3a, 0x62, 0x37, 0x30, 0xae,
//         0xad, 0x3e, 0x3d, 0xaa, 0xee, 0x9c, 0x60, 0x73, 0x1d, 0xb0, 0x5b, 0xe8, 0x62, 0x1c, 0x4b,
//         0xeb, 0x38,
//     ],
//     gy: [
//         0xd4, 0x81, 0x40, 0xd9, 0x50, 0xe2, 0x57, 0x7b, 0x26, 0xee, 0xb7, 0x41, 0xe7, 0xc6, 0x14,
//         0xe2, 0x24, 0xb7, 0xbd, 0xc9, 0x03, 0xf2, 0x9a, 0x28, 0xa8, 0x3c, 0xc8, 0x10, 0x11, 0x14,
//         0x5e, 0x06,
//     ],
// };

#[no_mangle]
pub extern "C" fn calculate_median(
    ciphertext: *mut u8,
    len: usize,
    median: &mut i64,
) -> sgx_status_t {
    let cipertext_slice: &mut [u8] = unsafe { slice::from_raw_parts_mut(ciphertext, len) };
    let mut data = decrypt_data(cipertext_slice).unwrap();

    data.sort_unstable();
    let mid = data.len() / 2;

    *median = data[mid];

    sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub extern "C" fn enclave_create_report(
    p_qe3_target: &sgx_target_info_t,
    p_data: &sgx_report_data_t,
    p_report: &mut sgx_report_t,
) -> u32 {
    match rsgx_create_report(p_qe3_target, p_data) {
        Ok(report) => {
            *p_report = report;
            0
        }
        Err(x) => {
            println!("rsgx_create_report failed! {:?}", x);
            x as u32
        }
    }
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

fn decrypt_data(ciphertext: &[u8]) -> Result<Vec<i64>, sgx_status_t> {
    let mut keyvec: Vec<u8> = Vec::new();

    let key_json_str = match SgxFile::open(KEYFILE) {
        Ok(mut f) => match f.read_to_end(&mut keyvec) {
            Ok(len) => {
                println!("Read {} bytes from Key file", len);
                std::str::from_utf8(&keyvec).unwrap()
            }
            Err(x) => {
                println!("Read keyfile failed {}", x);
                return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
            }
        },
        Err(x) => {
            println!("get_sealed_pcl_key cannot open keyfile, please check that key is provisioned successfully! {}", x);
            return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
        }
    };

    let mut plaintext = Vec::<u8>::new();

    let rsa_keypair: Rsa3072KeyPair = serde_json::from_str(&key_json_str).unwrap();

    rsa_keypair
        .decrypt_buffer(ciphertext, &mut plaintext)
        .unwrap();

    Ok(serde_json::from_slice(plaintext.as_slice()).unwrap())
}
