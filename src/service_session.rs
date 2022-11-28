macro_rules! create_session_guard {
    () => {
        pub struct Session;

        impl Session {
            pub fn new() -> crate::result::CtrResult<Self> {
                init()?;
                Ok(Self)
            }
        }

        impl Drop for Session {
            fn drop(&mut self) {
                exit()
            }
        }
    };
}

macro_rules! create_session_manager {
    ($init_impl:expr) => {
        mod service_session_manager {
            pub use super::*;
            pub use crate::{result::CtrResult, Handle};
            pub use core::{
                mem::ManuallyDrop,
                sync::atomic::{AtomicU32, Ordering},
            };

            static HANDLE: AtomicU32 = AtomicU32::new(0);
            static REF_COUNT: AtomicU32 = AtomicU32::new(0);

            pub fn get_handle() -> u32 {
                HANDLE.load(Ordering::Relaxed)
            }

            fn init_impl() -> CtrResult {
                let handle = { $init_impl };
                let dropped_handle = ManuallyDrop::new(handle);
                let raw_handle = unsafe { dropped_handle.get_raw() };
                HANDLE.store(raw_handle, Ordering::Relaxed);
                Ok(())
            }

            pub fn init() -> CtrResult {
                let previous_refs = REF_COUNT.fetch_add(1, Ordering::Relaxed);
                // if current_refs > 1
                if previous_refs > 0 {
                    return Ok(());
                }

                let result = init_impl();

                if result.is_err() {
                    REF_COUNT.fetch_min(1, Ordering::Relaxed);
                }

                result
            }

            pub fn exit() {
                let previous_refs = REF_COUNT.fetch_sub(1, Ordering::Relaxed);
                // if current_refs > 0
                if previous_refs > 1 {
                    return;
                }

                let raw_handle = HANDLE.swap(0, Ordering::Relaxed);
                drop(Handle::from(raw_handle));
            }

            crate::service_session::create_session_guard!();
        }

        pub use service_session_manager::{exit, get_handle, init, Session};
    };
    (ctru_sys, $init_impl:expr, $exit_impl:expr) => {
        mod service_session_manager {
            #[ctr_macros::hos]
            pub fn init() -> crate::result::CtrResult {
                let result = { $init_impl };
                crate::result::parse_result(result)
            }

            #[ctr_macros::hos]
            pub fn exit() {
                $exit_impl
            }

            crate::service_session::create_session_guard!();
        }

        pub use service_session_manager::{exit, init, Session};
    };
}

macro_rules! session {
    ($module_name:ident) => {
        let _session = crate::$module_name::Session::new()?;
    };
}

pub(crate) use create_session_guard;
pub(crate) use create_session_manager;
pub(crate) use session;
