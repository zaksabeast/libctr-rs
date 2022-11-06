/*
    Thanks to 3dbrew (https://www.3dbrew.org/wiki/Error_codes)
    and Citra (https://github.com/citra-emu/citra/blob/master/src/core/hle/result.h)
    for the documentation
*/

use core::convert::TryFrom;
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum KnownErrorSummary {
    Success = 0,
    NothingHappened = 1,
    WouldBlock = 2,
    OutOfResource = 3,
    NotFound = 4,
    InvalidState = 5,
    NotSupported = 6,
    InvalidArgument = 7,
    WrongArgument = 8,
    Canceled = 9,
    StatusChanged = 10,
    Internal = 11,
    InvalidResult = 63,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum ErrorSummary {
    Known(KnownErrorSummary),
    Other(u32),
}

impl From<KnownErrorSummary> for ErrorSummary {
    fn from(known: KnownErrorSummary) -> Self {
        Self::Known(known)
    }
}

impl From<u32> for ErrorSummary {
    fn from(raw: u32) -> Self {
        match KnownErrorSummary::try_from(raw) {
            Ok(known) => ErrorSummary::Known(known),
            Err(_) => ErrorSummary::Other(raw),
        }
    }
}

impl From<ErrorSummary> for u32 {
    fn from(val: ErrorSummary) -> Self {
        val.into()
    }
}
