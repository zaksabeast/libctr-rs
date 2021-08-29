use super::res::{CtrResult, GenericResultCode};
use safe_transmute::{Error, TriviallyTransmutable};

pub fn transmute_many_pedantic<T: safe_transmute::TriviallyTransmutable>(
    bytes: &[u8],
) -> CtrResult<&[T]> {
    safe_transmute::transmute_many_pedantic(bytes).map_err(|err| match err {
        Error::Guard(_) => GenericResultCode::InvalidSize.into(),
        Error::Unaligned(_) => GenericResultCode::AlignmentError.into(),
        Error::InvalidValue => GenericResultCode::InvalidValue.into(),
    })
}

pub fn transmute_many_pedantic_mut<T: TriviallyTransmutable>(
    bytes: &mut [u8],
) -> CtrResult<&mut [T]> {
    safe_transmute::transmute_many_pedantic_mut(bytes).map_err(|err| match err {
        Error::Guard(_) => GenericResultCode::InvalidSize.into(),
        Error::Unaligned(_) => GenericResultCode::AlignmentError.into(),
        Error::InvalidValue => GenericResultCode::InvalidValue.into(),
    })
}

pub fn transmute_one_pedantic<T: TriviallyTransmutable>(bytes: &[u8]) -> CtrResult<T> {
    safe_transmute::transmute_one_pedantic(&bytes).map_err(|err| match err {
        safe_transmute::Error::Guard(_) => GenericResultCode::InvalidSize.into(),
        safe_transmute::Error::Unaligned(_) => GenericResultCode::AlignmentError.into(),
        safe_transmute::Error::InvalidValue => GenericResultCode::InvalidValue.into(),
    })
}
