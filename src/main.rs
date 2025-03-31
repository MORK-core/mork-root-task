#![no_std]
#![no_main]
#![feature(linkage)]

use mork_common::mork_user_log;

mod hal;
mod lang_item;

#[unsafe(no_mangle)]
pub fn main() -> () {
    mork_user_lib::log_init();
    mork_user_log!(info, "Hello, world!");
    loop {

    }
}