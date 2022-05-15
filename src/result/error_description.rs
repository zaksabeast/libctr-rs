/*
    Thanks to 3dbrew (https://www.3dbrew.org/wiki/Error_codes)
    and Citra (https://github.com/citra-emu/citra/blob/master/src/core/hle/result.h)
    for the documentation
*/

use core::convert::TryFrom;
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum KnownErrorDescription {
    Success = 0,
    InvalidSection = 1000,
    TooLarge = 1001,
    NotAuthorized = 1002,
    AlreadyDone = 1003,
    InvalidSize = 1004,
    InvalidEnumValue = 1005,
    InvalidCombination = 1006,
    NoData = 1007,
    Busy = 1008,
    MisalignedAddress = 1009,
    MisalignedSize = 1010,
    OutOfMemory = 1011,
    NotImplemented = 1012,
    InvalidAddress = 1013,
    InvalidPointer = 1014,
    InvalidHandle = 1015,
    NotInitialized = 1016,
    AlreadyInitialized = 1017,
    NotFound = 1018,
    CancelRequested = 1019,
    AlreadyExists = 1020,
    OutOfRange = 1021,
    Timeout = 1022,
    InvalidResultValue = 1023,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ErrorDescription {
    Known(KnownErrorDescription),
    Other(u32),
}

impl From<KnownErrorDescription> for ErrorDescription {
    fn from(known: KnownErrorDescription) -> Self {
        Self::Known(known)
    }
}

impl From<u32> for ErrorDescription {
    fn from(raw: u32) -> Self {
        match KnownErrorDescription::try_from(raw) {
            Ok(known) => ErrorDescription::Known(known),
            Err(_) => ErrorDescription::Other(raw),
        }
    }
}

impl From<ErrorDescription> for u32 {
    fn from(val: ErrorDescription) -> Self {
        val.into()
    }
}
