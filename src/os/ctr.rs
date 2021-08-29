use ctru_sys::osGetTime;

/// Returns the OS time in milliseconds.
pub fn get_time() -> u64 {
    unsafe { osGetTime() }
}
