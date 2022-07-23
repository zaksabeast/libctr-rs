use std::ffi::{CString, NulError};
use crate::res::{CtrResult, GenericResultCode};
use std::str;

pub fn parse_result(result: Result<CString, NulError>) -> CtrResult<CString> {
    match result {
        Ok(cstring) => Ok(cstring),
        Err(_) => Err(GenericResultCode::NulError.into()),
    }
}

pub fn parse_null_terminated_str(bytes: &[u8]) -> &str {
    str::from_utf8(bytes)
        .unwrap_or("")
        .split(char::from(0))
        .next()
        .unwrap_or("")
}

#[cfg(test)]
mod test {
    use super::*;

    mod parse_null_terminated_str {
        use super::*;

        #[test]
        fn should_return_a_str() {
            let bytes = [0x74, 0x65, 0x73, 0x74];
            let result = parse_null_terminated_str(&bytes);
            assert_eq!(result, "test")
        }

        #[test]
        fn should_ignore_bytes_after_a_null_byte() {
            let bytes = [0x74, 0x65, 0x73, 0x00, 0x74];
            let result = parse_null_terminated_str(&bytes);
            assert_eq!(result, "tes")
        }

        #[test]
        fn should_return_an_empty_string_if_the_bytes_have_an_invalid_utf8_character() {
            let bytes = [0x74, 0x9f, 0x73, 0x74];
            let result = parse_null_terminated_str(&bytes);
            assert_eq!(result, "")
        }

        #[test]
        fn should_return_an_empty_string_if_the_bytes_start_with_a_null_terminator() {
            let bytes = [0x00, 0x74, 0x65, 0x73, 0x74];
            let result = parse_null_terminated_str(&bytes);
            assert_eq!(result, "")
        }

        #[test]
        fn should_return_an_empty_string_if_the_bytes_are_empty() {
            let bytes = [];
            let result = parse_null_terminated_str(&bytes);
            assert_eq!(result, "")
        }
    }
}
