use darling::FromMeta;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, ItemFn};

#[derive(Debug, FromMeta)]
pub struct MacroArgs {
    heap_byte_size: usize,
}

pub fn impl_ctr_start(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr_args = parse_macro_input!(attr as AttributeArgs);
    let main_func_def = parse_macro_input!(item as ItemFn);
    let main_func_ident = &main_func_def.sig.ident;

    let MacroArgs { heap_byte_size } = match MacroArgs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(e.write_errors());
        }
    };

    quote! {
        #[doc(hidden)]
        #main_func_def

        /// Called before main to initialize the system.
        /// Used by 3ds toolchain.
        #[doc(hidden)]
        #[no_mangle]
        pub extern "C" fn initSystem() {
            // This is safe because we're only supposed to use this one time
            // while initializing the system, which is happening right here.
            unsafe { ctr::allocator::init_heap(#heap_byte_size) };

            loop {
                match ctr::srv::init() {
                    Ok(_) => break,
                    Err(error_code) => {
                        if error_code != 0xd88007fa {
                            panic!();
                        }
                    }
                };

                ctr::svc::sleep_thread(500000i64);
            }
        }

        #[cfg(not(test))]
        #[doc(hidden)]
        #[panic_handler]
        fn panic(panic: &core::panic::PanicInfo<'_>) -> ! {
            if let Some(location) = panic.location() {
                let file = location.file();
                let slice = &file[file.len() - 7..];

                // Since we're about to break, storing a few u32s in these registers won't break us further.
                // In the future it might be helpful to disable this for release builds.
                unsafe {
                    // r9 and r10 aren't used as frequently as the lower registers, so in most situations
                    // we'll get more useful information by storing the last 4 characters of the file name
                    // and the line number where we broke.
                    let partial_file_name = *(slice.as_ptr() as *const u32);
                    core::arch::asm!("mov r9, {}", in(reg) partial_file_name);
                    core::arch::asm!("mov r10, {}", in(reg) location.line());
                }
            }

            ctr::svc::break_execution(ctr::svc::UserBreakType::Panic)
        }

        #[cfg(not(test))]
        #[doc(hidden)]
        #[no_mangle]
        pub extern "C" fn abort() -> ! {
            panic!()
        }

        #[doc(hidden)]
        #[start]
        fn __ctr_main(_argc: isize, _argv: *const *const u8) -> isize {
            #main_func_ident();
            ctr::svc::exit_process();
        }
    }
    .into()
}
