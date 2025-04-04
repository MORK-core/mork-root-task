#![no_std]
#![no_main]
#![feature(linkage)]

use mork_common::constants::CNodeSlot;
use mork_common::mork_user_log;
use mork_user_lib::mork_task::mork_thread_suspend;
use mork_user_lib::mork_tls::tls_init;
use mork_user_lib::mork_ipc_buffer::{ipc_buffer_init, with_ipc_buffer, with_ipc_buffer_mut};
extern crate alloc;
mod hal;
mod lang_item;
mod heap;

const MAIN_IPC_BUFFER_ADDR: usize = 0x1000_0000;

#[unsafe(no_mangle)]
pub fn main() -> () {
    mork_user_lib::log_init();

    mork_user_log!(info, "Hello, world!");

    heap::init();

    if let Err(_) = tls_init() {
        mork_user_log!(error, "Failed to initialize TLS!");
        mork_thread_suspend(CNodeSlot::CapInitThread as usize).unwrap();
    }

    mork_user_log!(info, "Successfully initialized TLS!");

    if let Err(_) = ipc_buffer_init(CNodeSlot::CapInitThread as usize, MAIN_IPC_BUFFER_ADDR) {
        mork_user_log!(error, "Failed to initialize IPC context!");
        mork_thread_suspend(CNodeSlot::CapInitThread as usize).unwrap();
    }

    with_ipc_buffer_mut(|buffer| {
        buffer.user_data = 100;
    });

    with_ipc_buffer(|buffer| {
        mork_user_log!(debug, "ipc buffer user data: {}", buffer.user_data);
    });

    mork_thread_suspend(CNodeSlot::CapInitThread as usize).unwrap();
}