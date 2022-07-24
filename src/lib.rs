mod handle;
pub use handle::*;

mod log;
pub use log::*;

mod process;
pub use process::*;

pub mod result;
pub use result as res;

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
#[cfg(target_os = "horizon")]
use ctru_sys::PrintConsole;

extern "C" fn services_deinit() {
    unsafe {
        ctru_sys::psExit();
    }
}

/// Call this somewhere to force Rust to link some required crates
/// This is also a setup for some crate integration only available at runtime
///
/// See <https://github.com/rust-lang/rust/issues/47384>
#[cfg(target_os = "horizon")]
pub fn init() {
    linker_fix_3ds::init();
    pthread_3ds::init();

    #[cfg(not(test))]
    panic_hook_setup();

    // Initialize the PS service for random data generation
    unsafe {
        let ps_ret = ctru_sys::psInit();
        if ctru_sys::R_FAILED(ps_ret) {
            panic!("Failed to initialize random data generation: {:?}", ps_ret)
        }

        // Setup the deconstruction at the program's end
        libc::atexit(services_deinit);
    }
}

#[cfg(not(test))]
static mut EMPTY_CONSOLE: PrintConsole = unsafe { const_zero::const_zero!(PrintConsole) };

#[cfg(not(test))]
fn console_exists() -> bool {
    unsafe {
        let current_console = ctru_sys::consoleSelect(&mut EMPTY_CONSOLE);
        let res = (*current_console).consoleInitialised;
        ctru_sys::consoleSelect(current_console);
        res
    }
}

#[cfg(not(test))]
fn panic_hook_setup() {
    use std::panic::PanicInfo;

    let main_thread = std::thread::current().id();

    // Panic Hook setup
    let default_hook = std::panic::take_hook();
    let new_hook = Box::new(move |info: &PanicInfo| {
        default_hook(info);

        // Only for panics in the main thread
        if main_thread == std::thread::current().id() && console_exists() {
            println!("\nPress SELECT to exit the software");

            match hid::init() {
                Ok(_) => loop {
                    hid::scan_input();
                    let keys = hid::keys_down();
                    if (keys & hid::Button::Select) != 0 {
                        break;
                    }
                },
                Err(e) => println!(
                    "Error while initializing Hid controller during panic: {:?}",
                    e
                ),
            }
        }
    });
    std::panic::set_hook(new_hook);
}
