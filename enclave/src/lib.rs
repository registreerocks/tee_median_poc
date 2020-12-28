#![crate_name = "tee_median"]
#![crate_type = "staticlib"]
#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

#[cfg(not(target_env = "sgx"))]
extern crate sgx_tstd as std;
extern crate sgx_types;

use sgx_types::*;
use std::slice;

#[no_mangle]
pub extern "C" fn calculate_median(slice: *mut i64, len: usize, median: &mut i64) -> sgx_status_t {
    let data: &mut [i64] = unsafe { slice::from_raw_parts_mut(slice, len) };

    data.sort_unstable();
    let mid = data.len() / 2;

    *median = data[mid];

    sgx_status_t::SGX_SUCCESS
}
