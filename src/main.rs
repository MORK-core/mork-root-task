#![no_std]
#![no_main]
#![feature(linkage)]

mod hal;
mod lang_item;

#[unsafe(no_mangle)]
fn main() -> () {
    panic!("Cannot find main!");
}