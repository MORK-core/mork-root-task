#![no_std]
#![no_main]
#![feature(linkage)]

use mork_common::constants::CNodeSlot;
use mork_common::mork_user_log;
use mork_user_lib::mork_task::mork_thread_suspend;
use crate::ipc_buffer::get_ipc_buffer;

extern crate alloc;
mod hal;
mod lang_item;
mod heap;
mod ipc_buffer;

#[unsafe(no_mangle)]
pub fn main() -> () {
    mork_user_lib::log_init();

    mork_user_log!(info, "Hello, world!");

    heap::init();
    if let Err(()) = ipc_buffer::init() {
        mork_user_log!(error, "Failed to initialize IPC context!");
        mork_thread_suspend(CNodeSlot::CapInitThread as usize).unwrap();
    }

    let ipc_buffer = get_ipc_buffer();
    ipc_buffer.user_data = 100;
    mork_user_log!(debug, "ipc buffer user data: {}", ipc_buffer.user_data);

    mork_thread_suspend(CNodeSlot::CapInitThread as usize).unwrap();
}