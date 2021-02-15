#[cfg_attr(sgx_mode = "SW", path = "quotes_sim.rs")]
#[cfg_attr(not(sgx_mode = "SW"), path = "quotes.rs")]
mod quotes;

mod quote_helpers;

extern crate serde;
extern crate serde_json;
extern crate sgx_crypto_helper;
extern crate sgx_types;
extern crate sgx_urts;
use serde_json::Result;
use sgx_types::*;
use sgx_urts::SgxEnclave;
use std::fs;

use sgx_crypto_helper::rsa3072::Rsa3072PubKey;
static ENCLAVE_FILE: &'static str = "enclave.signed.so";

extern "C" {
    fn calculate_median(
        eid: sgx_enclave_id_t,
        retval: *mut sgx_status_t,
        ciphertext: *mut u8,
        len: usize,
        median: *mut i64,
    ) -> sgx_status_t;

    fn create_keypair(
        eid: sgx_enclave_id_t,
        retval: *mut sgx_status_t,
        pubkey: *mut Rsa3072PubKey,
    ) -> sgx_status_t;
}

fn init_enclave() -> SgxResult<SgxEnclave> {
    let mut launch_token: sgx_launch_token_t = [0; 1024];
    let mut launch_token_updated: i32 = 0;
    // call sgx_create_enclave to initialize an enclave instance
    // Debug Support: set 2nd parameter to 1
    let debug = 1;
    let mut misc_attr = sgx_misc_attribute_t {
        secs_attr: sgx_attributes_t { flags: 0, xfrm: 0 },
        misc_select: 0,
    };
    SgxEnclave::create(
        ENCLAVE_FILE,
        debug,
        &mut launch_token,
        &mut launch_token_updated,
        &mut misc_attr,
    )
}

fn main() {
    let enclave = match init_enclave() {
        Ok(r) => {
            println!("[+] Init Enclave Successful {}!", r.geteid());
            r
        }
        Err(x) => {
            println!("[-] Init Enclave Failed {}!", x.as_str());
            return;
        }
    };
    let quote = quotes::get_quote(&enclave).unwrap();
    let maa_result = quote_helpers::parse_quote(quote);
    let maa_json = serde_json::to_string(&maa_result).unwrap();

    println!("{:?}", maa_json);
    fs::write("maa-data.json", maa_json);

    let slice: &mut [i64] = &mut [1, 2, 5, 3, 6];

    let mut c_retval = sgx_status_t::SGX_SUCCESS;
    let mut pubkey = Rsa3072PubKey::default();
    unsafe {
        create_keypair(enclave.geteid(), &mut c_retval, &mut pubkey);
    }
    let mut ciphertext = Vec::<u8>::new();
    let plaintext = serde_json::to_string(slice).unwrap();

    pubkey.encrypt_buffer(plaintext.as_bytes(), &mut ciphertext);

    let median: &mut i64 = &mut 0;

    let mut retval = sgx_status_t::SGX_SUCCESS;
    let result = unsafe {
        calculate_median(
            enclave.geteid(),
            &mut retval,
            ciphertext.as_mut_ptr(),
            ciphertext.len(),
            median,
        )
    };

    match result {
        sgx_status_t::SGX_SUCCESS => {
            println!("[+] calculate_median success. result: {}", median);
        }
        _ => {
            println!("[-] ECALL Enclave Failed {}!", result.as_str());
        }
    }
    enclave.destroy();
}
