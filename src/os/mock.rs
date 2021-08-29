#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn get_time() -> u64 {
    0
}
