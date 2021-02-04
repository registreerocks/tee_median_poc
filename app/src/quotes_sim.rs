extern crate data_encoding;
extern crate itertools;
extern crate libloading;
extern crate sgx_types;
extern crate sgx_urts;
use self::data_encoding::HEXUPPER;
use self::itertools::*;
use sgx_types::*;
use sgx_urts::SgxEnclave;
const QUOTE_RESULT: &'static str = include_str!("quote_result.txt");
extern "C" {
    fn enclave_create_report(
        eid: sgx_enclave_id_t,
        retval: *mut i32,
        p_qe3_target: &sgx_target_info_t,
        p_data: &sgx_report_data_t,
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
    p_data.d = *b"8cfa2d95950554edff7b0a5767b9a6efd22aed9dc42f893dbbfc2306fc4da967";

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

    let quote_size = std::mem::size_of::<sgx_target_info_t>();
    let mut v: Vec<u8> = vec![0; quote_size];
    unsafe {
        std::ptr::copy_nonoverlapping(
            &ti as *const sgx_target_info_t as *const u8,
            v.as_mut_ptr() as *mut u8,
            quote_size,
        );
    }

    let app_report: sgx_report_t = if let Some(r) = create_app_enclave_report(enclave, &ti) {
        println!("succeed!\nStep2: Call create_app_report:");
        r
    } else {
        println!("\nCall to create_app_report() failed\n");
        return None;
    };

    println!("NOTE: Quoting functionality did not run, using simulated flow.");

    println!(
        "app_report.body.mr_enclave = {:02x}",
        app_report.body.mr_enclave.m.iter().format("")
    );
    println!(
        "app_report.body.mr_signer = {:02x}",
        app_report.body.mr_signer.m.iter().format("")
    );
    println!(
        "app_report.body.cpu_svn = {:02x}",
        app_report.body.cpu_svn.svn.iter().format("")
    );
    println!(
        "app_report.body.misc_select = {:08x}",
        app_report.body.misc_select
    );
    println!(
        "app_report.key_id = {:02x}",
        app_report.key_id.id.iter().format("")
    );
    println!("app_report.mac = {:02x}", app_report.mac.iter().format(""));

    println!(
        "app_report.body.report_data.d = {:02x}",
        app_report.body.report_data.d.iter().format("")
    );

    println!(
        "app_report.body.report_data.d = {:?}",
        HEXUPPER.decode(&app_report.body.report_data.d),
    );

    let mut quote_size: u32 = 0;
    let decoded = HEXUPPER.decode(&QUOTE_RESULT.as_bytes()[0..QUOTE_RESULT.len() - 1]);
    // println!("{:?}", decoded);
    // let mut quote_vec: Vec<u8> = vec![0; decoded.unwrap()];
    Some(decoded.unwrap())
}
