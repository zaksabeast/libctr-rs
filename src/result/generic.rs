/*
    Thanks to 3dbrew (https://www.3dbrew.org/wiki/Error_codes)
    and Citra (https://github.com/citra-emu/citra/blob/master/src/core/hle/result.h)
    for the documentation
*/

use super::ResultCode;
use num_enum::IntoPrimitive;

#[derive(Clone, Copy, Debug, IntoPrimitive, PartialEq, Eq)]
#[repr(u32)]
/// Generic convenience error codes
pub enum GenericResultCode {
    NoArchive = 0xfffffffe,
    InvalidString = 0xc0de001,
    CheckedAdd = 0xc0de002,
    TryFromInt = 0xc0de003,
    TryFromBytes = 0xc0de004,
    NulError = 0xc0de005,
    OutOfSpace = 0xc0de006,
    InvalidBufferRights = 0xc0de007,
    AlignmentError = 0xc0de008,
    InvalidSize = 0xc0de009,
    InvalidValue = 0xc0de00a,
    InvalidPointer = 0xc0de00b,
    InvalidCommand = 0xd900182f,
}

impl GenericResultCode {
    // Convenience method for ambiguious convertions
    pub fn into_result_code(self) -> ResultCode {
        self.into()
    }
}
