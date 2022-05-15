/*
    Thanks to 3dbrew (https://www.3dbrew.org/wiki/Error_codes)
    and Citra (https://github.com/citra-emu/citra/blob/master/src/core/hle/result.h)
    for the documentation
*/

use super::{ErrorDescription, ErrorLevel, ErrorModule, ErrorSummary, GenericResultCode};

pub type CtrResult<T = ()> = Result<T, ResultCode>;

pub fn parse_result(result_code: i32) -> CtrResult {
    ResultCode::from(result_code).into_result()
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResultCode(u32);

impl ResultCode {
    pub fn new(
        description: impl Into<ErrorDescription>,
        level: impl Into<ErrorLevel>,
        summary: impl Into<ErrorSummary>,
        module: impl Into<ErrorModule>,
    ) -> Self {
        let raw = (u32::from(description.into()) & 0x3ff)
            | ((u32::from(summary.into()) & 0xff) << 10)
            | ((u32::from(module.into()) & 0x3f) << 21)
            | ((u32::from(level.into()) & 0x1f) << 27);
        Self(raw)
    }

    pub fn new_from_raw(raw: u32) -> Self {
        Self(raw)
    }

    pub fn success() -> Self {
        Self(0)
    }

    pub fn into_result(self) -> CtrResult {
        if self.get_is_success() {
            Ok(())
        } else {
            Err(self)
        }
    }

    pub fn into_raw(self) -> u32 {
        self.into()
    }

    pub fn get_description(&self) -> ErrorDescription {
        (self.0 & 0x3ff).into()
    }

    pub fn get_module(&self) -> ErrorModule {
        ((self.0 >> 10) & 0xff).into()
    }

    pub fn get_summary(&self) -> ErrorSummary {
        ((self.0 >> 21) & 0x3f).into()
    }

    pub fn get_level(&self) -> ErrorLevel {
        ((self.0 >> 27) & 0x1f).into()
    }

    pub fn get_is_error(&self) -> bool {
        ((self.0 >> 31) & 1) != 0
    }

    pub fn get_is_success(&self) -> bool {
        !self.get_is_error()
    }
}

impl From<ResultCode> for u32 {
    fn from(result_code: ResultCode) -> Self {
        result_code.0
    }
}

impl From<u32> for ResultCode {
    fn from(result_code: u32) -> Self {
        ResultCode(result_code)
    }
}

// This is useful for libctru bindings
impl From<i32> for ResultCode {
    fn from(result_code: i32) -> Self {
        ResultCode(result_code as u32)
    }
}

impl From<GenericResultCode> for ResultCode {
    fn from(result_code: GenericResultCode) -> Self {
        ResultCode(result_code.into())
    }
}

impl PartialEq<u32> for ResultCode {
    fn eq(&self, other: &u32) -> bool {
        self.0 == *other
    }
}

impl PartialEq<ResultCode> for u32 {
    fn eq(&self, other: &ResultCode) -> bool {
        *self == other.0
    }
}

impl PartialEq<GenericResultCode> for ResultCode {
    fn eq(&self, other: &GenericResultCode) -> bool {
        self.0 == u32::from(*other)
    }
}

impl PartialEq<ResultCode> for GenericResultCode {
    fn eq(&self, other: &ResultCode) -> bool {
        u32::from(*self) == other.0
    }
}
