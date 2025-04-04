use mork_common::constants::{CNodeSlot, ObjectType};
use mork_common::mork_user_log;
use mork_common::syscall::ipc_buffer::IPCBuffer;
use mork_common::types::{JustResult, VMRights};
use mork_user_lib::mork_alloc::mork_alloc_object;
use mork_user_lib::mork_mm::mork_map_frame_anyway;

const IPC_BUFFER_ADDR: usize = 0x10000000;

pub fn init() -> JustResult {
    match mork_alloc_object(CNodeSlot::CapInitCNode as usize, ObjectType::Frame) {
        Ok(frame_handler) => {
            mork_user_log!(debug, "success to allocate memory: {:?}", frame_handler);
            let vm_rights = VMRights::R | VMRights::W;
            match mork_map_frame_anyway(
                CNodeSlot::CapInitCNode as usize,
                CNodeSlot::CapInitVSpace as usize,
                frame_handler,
                IPC_BUFFER_ADDR,
                vm_rights
            ) {
                Ok(_) => {
                    mork_user_log!(debug, "map ipc buffer successfully!");
                    Ok(())
                }
                Err(resp) => {
                    mork_user_log!(error, "fail to map frame: {:?}", resp);
                    Err(())
                }
            }
        }
        Err(resp) => {
            mork_user_log!(error, "fail to allocate memory: {:?}", resp);
            Err(())
        }
    }
}

pub fn get_ipc_buffer() ->  &'static mut IPCBuffer {
    mork_user_log!(debug, "ipc buffer size: {}", size_of::<IPCBuffer>());
    unsafe {
        &mut *(IPC_BUFFER_ADDR as *mut IPCBuffer)
    }
}