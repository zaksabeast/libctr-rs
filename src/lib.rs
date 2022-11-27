#![no_std]
#![feature(alloc_error_handler)]
#![cfg_attr(not(target_os = "horizon"), allow(unused))]

extern crate alloc;

pub mod allocator;
pub use allocator::*;

mod handle;
pub use handle::*;

mod log;
pub use log::*;

mod process;
pub use process::*;

pub mod result;
pub use result as res;
pub use result::error;

pub mod ac;
pub mod cfg;
pub mod frd;
pub mod fs;
pub mod hid;
pub mod http;
pub mod ipc;
pub mod memory;
pub mod ndm;
pub mod os;
pub mod pm_dbg;
pub mod ps;
pub mod ptm;
pub mod srv;
pub mod svc;
pub mod sysmodule;
pub mod time;
pub mod utils;
pub use ctr_macros::*;
pub mod thread;
