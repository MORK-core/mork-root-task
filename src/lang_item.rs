use core::panic::PanicInfo;
use mork_common::constants::CNodeSlot;
use mork_common::mork_user_log;
use mork_user_lib::mork_task::mork_thread_suspend;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        mork_user_log!(error,
            "[user] Panicked at {}:{} {}",
            location.file(),
            location.line(),
            info.message()
        );
    } else {
        mork_user_log!(error, "[user] Panicked: {}", info.message());
    }
    mork_thread_suspend(CNodeSlot::CapInitThread as usize).unwrap();
    panic!()
}