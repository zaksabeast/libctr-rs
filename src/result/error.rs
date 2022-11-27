use super::{
    KnownErrorDescription, KnownErrorLevel, KnownErrorModule, KnownErrorSummary, ResultCode,
};

pub fn success() -> ResultCode {
    ResultCode::new_generic_description(KnownErrorDescription::Success)
}

pub fn invalid_section() -> ResultCode {
    ResultCode::new_generic_description(KnownErrorDescription::InvalidSection)
}

pub fn too_large() -> ResultCode {
    ResultCode::new_generic_description(KnownErrorDescription::TooLarge)
}

pub fn not_authorized() -> ResultCode {
    ResultCode::new_generic_description(KnownErrorDescription::NotAuthorized)
}

pub fn already_done() -> ResultCode {
    ResultCode::new_generic_description(KnownErrorDescription::AlreadyDone)
}

pub fn invalid_size() -> ResultCode {
    ResultCode::new_generic_description(KnownErrorDescription::InvalidSize)
}

pub fn invalid_enum_value() -> ResultCode {
    ResultCode::new_generic_description(KnownErrorDescription::InvalidEnumValue)
}

pub fn invalid_combination() -> ResultCode {
    ResultCode::new_generic_description(KnownErrorDescription::InvalidCombination)
}

pub fn no_data() -> ResultCode {
    ResultCode::new_generic_description(KnownErrorDescription::NoData)
}

pub fn busy() -> ResultCode {
    ResultCode::new_generic_description(KnownErrorDescription::Busy)
}

pub fn misaligned_address() -> ResultCode {
    ResultCode::new_generic_description(KnownErrorDescription::MisalignedAddress)
}

pub fn misaligned_size() -> ResultCode {
    ResultCode::new_generic_description(KnownErrorDescription::MisalignedSize)
}

pub fn out_of_memory() -> ResultCode {
    ResultCode::new_generic_description(KnownErrorDescription::OutOfMemory)
}

pub fn not_implemented() -> ResultCode {
    ResultCode::new_generic_description(KnownErrorDescription::NotImplemented)
}

pub fn invalid_address() -> ResultCode {
    ResultCode::new_generic_description(KnownErrorDescription::InvalidAddress)
}

pub fn invalid_pointer() -> ResultCode {
    ResultCode::new_generic_description(KnownErrorDescription::InvalidPointer)
}

pub fn invalid_handle() -> ResultCode {
    ResultCode::new_generic_description(KnownErrorDescription::InvalidHandle)
}

pub fn not_initialized() -> ResultCode {
    ResultCode::new_generic_description(KnownErrorDescription::NotInitialized)
}

pub fn already_initialized() -> ResultCode {
    ResultCode::new_generic_description(KnownErrorDescription::AlreadyInitialized)
}

pub fn not_found() -> ResultCode {
    ResultCode::new_generic_description(KnownErrorDescription::NotFound)
}

pub fn cancel_requested() -> ResultCode {
    ResultCode::new_generic_description(KnownErrorDescription::CancelRequested)
}

pub fn already_exists() -> ResultCode {
    ResultCode::new_generic_description(KnownErrorDescription::AlreadyExists)
}

pub fn out_of_range() -> ResultCode {
    ResultCode::new_generic_description(KnownErrorDescription::OutOfRange)
}

pub fn timeout() -> ResultCode {
    ResultCode::new_generic_description(KnownErrorDescription::Timeout)
}

pub fn invalid_result_value() -> ResultCode {
    ResultCode::new_generic_description(KnownErrorDescription::InvalidResultValue)
}

pub fn invalid_value() -> ResultCode {
    ResultCode::new_generic(
        KnownErrorDescription::InvalidResultValue,
        KnownErrorSummary::InvalidArgument,
    )
}

pub fn invalid_argument() -> ResultCode {
    ResultCode::new_generic(
        KnownErrorDescription::InvalidResultValue,
        KnownErrorSummary::InvalidArgument,
    )
}

pub fn invalid_command() -> ResultCode {
    // 0xd900182f
    ResultCode::new(
        KnownErrorDescription::InvalidCommand,
        KnownErrorLevel::Permanent,
        KnownErrorSummary::WrongArgument,
        KnownErrorModule::Os,
    )
}
