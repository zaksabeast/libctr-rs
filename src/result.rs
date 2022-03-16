/*
    Thanks to 3dbrew (https://www.3dbrew.org/wiki/Error_codes)
    and Citra (https://github.com/citra-emu/citra/blob/master/src/core/hle/result.h)
    for the documentation
*/

use core::convert::TryFrom;
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Clone, Copy, Debug, IntoPrimitive)]
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

#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum KnownErrorModule {
    Common = 0,
    Kernel = 1,
    Util = 2,
    FileServer = 3,
    LoaderServer = 4,
    Tcb = 5,
    Os = 6,
    Dbc = 7,
    Dmnt = 8,
    Pdn = 9,
    Gx = 10,
    I2c = 11,
    Gpio = 12,
    Dd = 13,
    Codec = 14,
    Spi = 15,
    Pxi = 16,
    Fs = 17,
    Di = 18,
    Hid = 19,
    Cam = 20,
    Pi = 21,
    Pm = 22,
    PmLow = 23,
    Fsi = 24,
    Srv = 25,
    Ndn = 26,
    Nwm = 27,
    Soc = 28,
    Ldr = 29,
    Acc = 30,
    RomFS = 31,
    Am = 32,
    Hio = 33,
    Updater = 34,
    Mic = 35,
    Fnd = 36,
    Mp = 37,
    Mpwl = 38,
    Ac = 39,
    Http = 40,
    Dsp = 41,
    Snd = 42,
    Dlp = 43,
    HioLow = 44,
    Csnd = 45,
    Ssl = 46,
    AmLow = 47,
    Nex = 48,
    Friends = 49,
    Rdt = 50,
    Applet = 51,
    Nim = 52,
    Ptm = 53,
    Midi = 54,
    Mc = 55,
    Swc = 56,
    FatFS = 57,
    Ngc = 58,
    Card = 59,
    Cardnor = 60,
    Sdmc = 61,
    Boss = 62,
    Dbm = 63,
    Config = 64,
    Ps = 65,
    Cec = 66,
    Ir = 67,
    Uds = 68,
    Pl = 69,
    Cup = 70,
    Gyroscope = 71,
    Mcu = 72,
    Ns = 73,
    News = 74,
    Ro = 75,
    Gd = 76,
    CardSPI = 77,
    Ec = 78,
    WebBrowser = 79,
    Test = 80,
    Enc = 81,
    Pia = 82,
    Act = 83,
    Vctl = 84,
    Olv = 85,
    Neia = 86,
    Npns = 87,
    Avd = 90,
    L2b = 91,
    Mvd = 92,
    Nfc = 93,
    Uart = 94,
    Spm = 95,
    Qtm = 96,
    Nfp = 97,
    Application = 254,
    InvalidResult = 255,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ErrorModule {
    Known(KnownErrorModule),
    Other(u32),
}

impl From<KnownErrorModule> for ErrorModule {
    fn from(known: KnownErrorModule) -> Self {
        Self::Known(known)
    }
}

impl From<u32> for ErrorModule {
    fn from(raw: u32) -> Self {
        match KnownErrorModule::try_from(raw) {
            Ok(known) => ErrorModule::Known(known),
            Err(_) => ErrorModule::Other(raw),
        }
    }
}

impl From<ErrorModule> for u32 {
    fn from(val: ErrorModule) -> Self {
        val.into()
    }
}

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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

#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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

pub type CtrResult<T = ()> = Result<T, ResultCode>;

pub fn parse_result(result_code: i32) -> CtrResult {
    ResultCode::from(result_code).into_result()
}

#[derive(Clone, Copy, Debug)]
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

impl PartialEq<ResultCode> for ResultCode {
    fn eq(&self, other: &ResultCode) -> bool {
        self.0 == other.0
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
