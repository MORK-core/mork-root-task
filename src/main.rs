#![no_std]
#![no_main]
#![feature(linkage)]

use mork_common::constants::{CNodeSlot, ObjectType};
use mork_common::mork_user_log;
use mork_common::types::VMRights;
use mork_user_lib::mork_alloc::mork_alloc_object;
use mork_user_lib::mork_mm::mork_map_frame_anyway;
use mork_user_lib::mork_task::mork_thread_suspend;
extern crate alloc;
mod hal;
mod lang_item;
mod heap;

pub const IPC_BUFFER_ADDR: usize = 0x10000000;

#[unsafe(no_mangle)]
pub fn main() -> () {
    mork_user_lib::log_init();

    mork_user_log!(info, "Hello, world!");

    heap::init();
    match mork_alloc_object(CNodeSlot::CapInitCNode as usize, ObjectType::Frame) {
        Ok(frame_handler) => {
            mork_user_log!(info, "success to allocate memory: {:?}", frame_handler);
            let vm_rights = VMRights::R | VMRights::W;
            match mork_map_frame_anyway(
                CNodeSlot::CapInitCNode as usize,
                CNodeSlot::CapInitVSpace as usize,
                frame_handler,
                IPC_BUFFER_ADDR,
                vm_rights
            ) {
                Ok(_) => {}
                Err(resp) => {
                    mork_user_log!(error, "fail to map frame: {:?}", resp);
                    mork_thread_suspend(CNodeSlot::CapInitThread as usize).unwrap();
                }
            }

        }
        Err(resp) => {
            mork_user_log!(error, "fail to allocate memory: {:?}", resp);
            mork_thread_suspend(CNodeSlot::CapInitThread as usize).unwrap();
        }
    }
    mork_user_log!(info, "map ipc buffer successfully!");
    mork_thread_suspend(CNodeSlot::CapInitThread as usize).unwrap();

}