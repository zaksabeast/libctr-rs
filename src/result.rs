use core::convert::From;

pub type ResultCode = i32;

pub type CtrResult<T> = Result<T, ResultCode>;

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(i32)]
/// Provides names to custom error codes for common errors that might occur.
pub enum GenericResultCode {
    Success = 0,
    NoArchive = -2,
    InvalidString = 0xC0DE001,
    CheckedAdd = 0xC0DE002,
    TryFromInt = 0xC0DE003,
    TryFromBytes = 0xC0DE004,
    NulError = 0xC0DE005,
    OutOfSpace = 0xC0DE006,
    InvalidBufferRights = 0xC0DE007,
    AlignmentError = 0xC0DE008,
    InvalidSize = 0xC0DE009,
    InvalidValue = 0xC0DE00A,
    InvalidPointer = 0xC0DE00B,
    InvalidCommand = -0x26ffe7d1,
}

impl From<GenericResultCode> for i32 {
    fn from(error: GenericResultCode) -> Self {
        error as i32
    }
}

impl From<GenericResultCode> for u32 {
    fn from(error: GenericResultCode) -> Self {
        error as u32
    }
}

impl PartialEq<i32> for GenericResultCode {
    fn eq(&self, other: &i32) -> bool {
        *self as i32 == *other
    }
}

#[inline]
pub fn check_if_fail(result: ResultCode) -> bool {
    result < 0
}

#[inline]
pub fn check_if_success(result: ResultCode) -> bool {
    result >= 0
}

#[inline]
/// Parses a result code as a Result.
pub fn parse_result(result: ResultCode) -> CtrResult<ResultCode> {
    if check_if_success(result) {
        Ok(result)
    } else {
        Err(result)
    }
}
