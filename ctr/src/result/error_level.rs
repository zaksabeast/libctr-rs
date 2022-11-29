/*
    Thanks to 3dbrew (https://www.3dbrew.org/wiki/Error_codes)
    and Citra (https://github.com/citra-emu/citra/blob/master/src/core/hle/result.h)
    for the documentation
*/

use core::convert::TryFrom;
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum KnownErrorLevel {
    Success = 0,
    Info = 1,
    Status = 25,
    Temporary = 26,
    Permanent = 27,
    Usage = 28,
    Reinitialize = 29,
    Reset = 30,
    Fatal = 31,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ErrorLevel {
    Known(KnownErrorLevel),
    Other(u32),
}

impl From<KnownErrorLevel> for ErrorLevel {
    fn from(known: KnownErrorLevel) -> Self {
        Self::Known(known)
    }
}

impl From<u32> for ErrorLevel {
    fn from(raw: u32) -> Self {
        match KnownErrorLevel::try_from(raw) {
            Ok(known) => ErrorLevel::Known(known),
            Err(_) => ErrorLevel::Other(raw),
        }
    }
}

impl From<ErrorLevel> for u32 {
    fn from(val: ErrorLevel) -> Self {
        val.into()
    }
}
