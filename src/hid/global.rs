use super::InterfaceDevice;
use core::sync::atomic::{AtomicU32, Ordering};

static CURRENT_IO_BITS: AtomicU32 = AtomicU32::new(0);
static PREVIOUS_IO_BITS: AtomicU32 = AtomicU32::new(0);

/// A global interface to read 3ds buttons.
/// Must have access to the memory mapped IO at 0x1EC00000-0x1ECFFFFF.
pub struct Global();

impl InterfaceDevice for Global {
    #[ctr_macros::hos]
    fn scan_input() {
        // This is io mapped memory that should always be available
        let new_io_bits = unsafe { (0x1ec46000 as *mut u32).read() };
        let old_io_bits = CURRENT_IO_BITS.load(Ordering::Relaxed);

        CURRENT_IO_BITS.store((new_io_bits ^ 0xfff) & 0xfff, Ordering::Relaxed);
        PREVIOUS_IO_BITS.store(old_io_bits, Ordering::Relaxed);
    }

    fn get_io_bits() -> u32 {
        CURRENT_IO_BITS.load(Ordering::Relaxed)
    }

    fn get_previous_io_bits() -> u32 {
        PREVIOUS_IO_BITS.load(Ordering::Relaxed)
    }
}
