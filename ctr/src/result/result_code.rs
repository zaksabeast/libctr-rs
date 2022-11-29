/*
    Thanks to 3dbrew (https://www.3dbrew.org/wiki/Error_codes)
    and Citra (https://github.com/citra-emu/citra/blob/master/src/core/hle/result.h)
    for the documentation
*/

use super::{
    ErrorDescription, ErrorLevel, ErrorModule, ErrorSummary, KnownErrorLevel, KnownErrorModule,
    KnownErrorSummary,
};
use core::{fmt, mem};
use no_std_io::{EndianRead, EndianWrite, ReadOutput, Writer};

pub type CtrResult<T = ()> = Result<T, ResultCode>;

pub fn parse_result(raw_result_code: impl Into<ResultCode>) -> CtrResult {
    let result_code: ResultCode = raw_result_code.into();
    result_code.into_result()
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ResultCode {
    pub(super) raw: u32,
}

impl ResultCode {
    #[inline(always)]
    pub fn new(
        description: impl Into<u32>,
        level: impl Into<u32>,
        summary: impl Into<u32>,
        module: impl Into<u32>,
    ) -> Self {
        let raw = (description.into() & 0x3ff)
            | ((summary.into() & 0xff) << 10)
            | ((module.into() & 0x3f) << 21)
            | ((level.into() & 0x1f) << 27);
        Self { raw }
    }

    pub fn new_from_raw(raw: u32) -> Self {
        Self { raw }
    }

    #[inline(always)]
    pub fn new_generic(description: impl Into<u32>, summary: impl Into<u32>) -> Self {
        Self::new(
            description,
            KnownErrorLevel::Permanent,
            summary,
            KnownErrorModule::Common,
        )
    }

    #[inline(always)]
    pub fn new_generic_description(description: impl Into<u32>) -> Self {
        Self::new(
            description,
            KnownErrorLevel::Permanent,
            KnownErrorSummary::NotSupported,
            KnownErrorModule::Common,
        )
    }

    #[inline(always)]
    pub fn success() -> Self {
        Self { raw: 0 }
    }

    #[inline(always)]
    pub fn into_result(self) -> CtrResult {
        if self.is_success() {
            Ok(())
        } else {
            Err(self)
        }
    }

    #[inline(always)]
    pub fn into_raw(self) -> u32 {
        self.into()
    }

    #[inline(always)]
    pub fn description(&self) -> ErrorDescription {
        (self.raw & 0x3ff).into()
    }

    #[inline(always)]
    pub fn module(&self) -> ErrorModule {
        ((self.raw >> 10) & 0xff).into()
    }

    #[inline(always)]
    pub fn summary(&self) -> ErrorSummary {
        ((self.raw >> 21) & 0x3f).into()
    }

    #[inline(always)]
    pub fn level(&self) -> ErrorLevel {
        ((self.raw >> 27) & 0x1f).into()
    }

    #[inline(always)]
    pub fn is_error(&self) -> bool {
        ((self.raw >> 31) & 1) != 0
    }

    #[inline(always)]
    pub fn is_success(&self) -> bool {
        !self.is_error()
    }
}

impl fmt::Debug for ResultCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResultCode")
            .field("description", &self.description())
            .field("level", &self.level())
            .field("summary", &self.summary())
            .field("module", &self.module())
            .finish()
    }
}

impl From<ResultCode> for u32 {
    fn from(result_code: ResultCode) -> Self {
        result_code.raw
    }
}

impl From<u32> for ResultCode {
    fn from(result_code: u32) -> Self {
        ResultCode::new_from_raw(result_code)
    }
}

// This is useful for libctru bindings
impl From<i32> for ResultCode {
    fn from(result_code: i32) -> Self {
        ResultCode::new_from_raw(result_code as u32)
    }
}

impl PartialEq<u32> for ResultCode {
    fn eq(&self, other: &u32) -> bool {
        self.raw == *other
    }
}

impl PartialEq<ResultCode> for u32 {
    fn eq(&self, other: &ResultCode) -> bool {
        *self == other.raw
    }
}

impl EndianRead for ResultCode {
    fn try_read_le(bytes: &[u8]) -> Result<ReadOutput<Self>, no_std_io::Error> {
        Ok(u32::try_read_le(bytes)?.into_other())
    }

    fn try_read_be(_bytes: &[u8]) -> Result<ReadOutput<Self>, no_std_io::Error> {
        unimplemented!()
    }
}

impl EndianWrite for ResultCode {
    fn get_size(&self) -> usize {
        mem::size_of::<u32>()
    }

    fn try_write_le(&self, mut dst: &mut [u8]) -> Result<usize, no_std_io::Error> {
        dst.write_le(0, &self.raw)?;
        Ok(mem::size_of::<u32>())
    }

    fn try_write_be(&self, _dst: &mut [u8]) -> Result<usize, no_std_io::Error> {
        unimplemented!()
    }
}
