use crate::res::CtrResult;

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn init() -> CtrResult {
    Ok(())
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn exit() {}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn scan_input() {}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn keys_held() -> u32 {
    0
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn keys_down() -> u32 {
    0
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn keys_down_repeat() -> u32 {
    0
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn keys_up() -> u32 {
    0
}
