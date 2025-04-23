
#![no_std]
#![no_main]
#![feature(linkage)]
extern crate alloc;

use mork_common::mork_user_log;
use mork_user_lib::mork_shutdown;

mod auto_gen;
mod test_cases;

#[unsafe(no_mangle)]
pub fn main() {
    mork_user_log!(info, "hello from mork root task!");
    if let Err(resp) = test_cases::parse() {
        mork_user_log!(error, "fail to parse test cases: {:?}", resp);
        return;
    }
    if let Err(resp) = test_cases::run() {
        mork_user_log!(error, "fail to run test cases: {:?}", resp);
        return;
    }
    mork_user_log!(info, "****************");
    mork_user_log!(info, "all tests passed!");
    mork_shutdown();
}