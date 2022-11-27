/*
    Thanks to 3dbrew (https://www.3dbrew.org/wiki/Error_codes)
    and Citra (https://github.com/citra-emu/citra/blob/master/src/core/hle/result.h)
    for the documentation
*/

use core::convert::TryFrom;
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
