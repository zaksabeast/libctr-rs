use super::{error, ResultCode};
use alloc::{str::Utf8Error, string::FromUtf16Error};
use core::num::{ParseIntError, TryFromIntError};
use cstr_core::NulError;
use no_std_io::Error as NoStdIoError;

impl From<TryFromIntError> for ResultCode {
    fn from(_: TryFromIntError) -> Self {
        error::invalid_value()
    }
}

impl From<ParseIntError> for ResultCode {
    fn from(_: ParseIntError) -> Self {
        error::invalid_value()
    }
}

impl From<FromUtf16Error> for ResultCode {
    fn from(_: FromUtf16Error) -> Self {
        error::invalid_value()
    }
}

impl From<Utf8Error> for ResultCode {
    fn from(_: Utf8Error) -> Self {
        error::invalid_value()
    }
}

impl From<NulError> for ResultCode {
    fn from(_: NulError) -> Self {
        error::no_data()
    }
}

impl From<NoStdIoError> for ResultCode {
    fn from(error: NoStdIoError) -> Self {
        match error {
            NoStdIoError::InvalidAlignment { .. } => error::misaligned_address(),
            NoStdIoError::InvalidSize { .. } => error::invalid_size(),
            NoStdIoError::InvalidRead { .. } => error::invalid_value(),
            NoStdIoError::InvalidWrite { .. } => error::invalid_value(),
        }
    }
}
