
#![no_std]
#![no_main]
#![feature(linkage)]
extern crate alloc;

use mork_common::constants::CNodeSlot;
use mork_common::mork_user_log;
use mork_user_lib::mork_ipc_buffer::ipc_buffer_init;
use mork_user_lib::mork_tls::tls_init;

mod auto_gen;
mod test_cases;

const IPC_BUFFER_VADDR: usize = 0x2000_0000;

#[unsafe(no_mangle)]
pub fn main() {
    mork_user_log!(info, "hello from mork root task!");
    if let Err(()) = tls_init() {
        mork_user_log!(error, "mork-root-task failed to initialize TLS context!");
        return;
    }
    if let Err(()) = ipc_buffer_init(CNodeSlot::CapInitThread as usize, IPC_BUFFER_VADDR) {
        mork_user_log!(error, "mork-root-task ipc_buffer_init failed");
        return;
    }
    if let Err(resp) = test_cases::parse() {
        mork_user_log!(error, "fail to parse test cases: {:?}", resp);
        return;
    }
    if let Err(resp) = test_cases::run() {
        mork_user_log!(error, "fail to run test cases: {:?}", resp);
        return;
    }
}