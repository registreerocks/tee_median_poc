extern crate hex_literal;
extern crate itertools;
extern crate libloading;
extern crate sgx_types;
extern crate sgx_urts;
use self::hex_literal::*;
use self::itertools::*;
use sgx_types::*;
use sgx_urts::SgxEnclave;

extern "C" {
    fn enclave_create_report(
        eid: sgx_enclave_id_t,
        retval: *mut i32,
        p_qe3_target: &sgx_target_info_t,
        p_data: sgx_report_data_t,
        p_report: *mut sgx_report_t,
    ) -> sgx_status_t;
}

pub fn get_quote(enclave: &SgxEnclave) -> Option<Vec<u8>> {
    generate_quote(enclave)
}

fn create_app_enclave_report(
    enclave: &SgxEnclave,
    qe_ti: &sgx_target_info_t,
) -> Option<sgx_report_t> {
    let mut retval = 0;
    let mut ret_report: sgx_report_t = sgx_report_t::default();
    let mut p_data = sgx_report_data_t::default();
    p_data.d = [0; 64];
    p_data.d.copy_from_slice(
        "fd49c0425a1aeff9fe04025a5083fa0fc8392f50e7ab779d00d3451805c2b240".as_bytes(),
    );
    p_data.d = [0; 64];
    p_data.d.copy_from_slice(
        &[
            &hex!("fd49c0425a1aeff9fe04025a5083fa0fc8392f50e7ab779d00d3451805c2b240"),
            [0u8].repeat(32).as_slice(),
        ]
        .concat(),
    );

    let result = unsafe {
        enclave_create_report(
            enclave.geteid(),
            &mut retval,
            qe_ti,
            &p_data,
            &mut ret_report as *mut sgx_report_t,
        )
    };
    match result {
        sgx_status_t::SGX_SUCCESS => {}
        _ => {
            println!("[-] ECALL Enclave Failed {}!", result.as_str());
            return None;
        }
    }
    Some(ret_report)
}

fn generate_quote(enclave: &SgxEnclave) -> Option<Vec<u8>> {
    let mut ti: sgx_target_info_t = sgx_target_info_t::default();

    //let _l = libloading::Library::new("./libdcap_quoteprov.so.1").unwrap();
    println!("Step1: Call sgx_qe_get_target_info:");

    let qe3_ret = unsafe { sgx_qe_get_target_info(&mut ti as *mut _) };

    if qe3_ret != sgx_quote3_error_t::SGX_QL_SUCCESS {
        println!("Error in sgx_qe_get_target_info. {:?}\n", qe3_ret);
        return None;
    }

    //println!("target_info.mr_enclave = {:?}", ti.mr_enclave.m);
    //println!("target_info.config_id = {:02x}", ti.config_id.iter().format(" "));

    let quote_size = std::mem::size_of::<sgx_target_info_t>();
    let mut v: Vec<u8> = vec![0; quote_size];
    unsafe {
        std::ptr::copy_nonoverlapping(
            &ti as *const sgx_target_info_t as *const u8,
            v.as_mut_ptr() as *mut u8,
            quote_size,
        );
    }

    //println!("quote = {:?}", v);

    println!("succeed!\nStep2: Call create_app_report:");
    let app_report: sgx_report_t = if let Some(r) = create_app_enclave_report(enclave, &ti) {
        println!("succeed! \nStep3: Call sgx_qe_get_quote_size:");
        r
    } else {
        println!("\nCall to create_app_report() failed\n");
        return None;
    };

    //println!("app_report.body.cpu_svn = {:02x}", app_report.body.cpu_svn.svn.iter().format(""));
    //println!("app_report.body.misc_select = {:08x}", app_report.body.misc_select);
    //println!("app_report.key_id = {:02x}", app_report.key_id.id.iter().format(""));
    //println!("app_report.mac = {:02x}", app_report.mac.iter().format(""));

    println!(
        "app_report.body.mr_enclave = {:02x}",
        app_report.body.mr_enclave.m.iter().format("")
    );
    println!(
        "app_report.body.mr_signer = {:02x}",
        app_report.body.mr_signer.m.iter().format("")
    );

    let mut quote_size: u32 = 0;
    let qe3_ret = unsafe { sgx_qe_get_quote_size(&mut quote_size as _) };

    if qe3_ret != sgx_quote3_error_t::SGX_QL_SUCCESS {
        println!("Error in sgx_qe_get_quote_size . {:?}\n", qe3_ret);
        return None;
    }

    println!("succeed!");

    let mut quote_vec: Vec<u8> = vec![0; quote_size as usize];

    println!("\nStep4: Call sgx_qe_get_quote:");
    let qe3_ret =
        unsafe { sgx_qe_get_quote(&app_report as _, quote_size, quote_vec.as_mut_ptr() as _) };

    if qe3_ret != sgx_quote3_error_t::SGX_QL_SUCCESS {
        println!("Error in sgx_qe_get_quote. {:?}\n", qe3_ret);
        return None;
    }

    Some(quote_vec)
}
