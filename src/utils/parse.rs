use crate::res::{CtrResult, GenericResultCode};
use alloc::str;
use core::str::FromStr;

pub fn parse_num<T: FromStr>(num: &str) -> CtrResult<T> {
    num.parse()
        .map_err(|_| GenericResultCode::InvalidValue.into())
}

pub fn str_from_utf8(bytes: &[u8]) -> CtrResult<&str> {
    str::from_utf8(bytes).map_err(|_| GenericResultCode::InvalidString.into())
}

#[cfg(test)]
mod test {
    use super::*;

    mod parse_num {
        use super::*;

        #[test]
        fn should_parse_a_number_from_a_string() {
            let result: u64 = parse_num("12345").unwrap();
            assert_eq!(result, 12345u64);
        }

        #[test]
        fn should_return_an_error_if_the_string_is_an_invalid_number() {
            let result = parse_num::<u64>("abcd").unwrap_err();
            assert_eq!(result, GenericResultCode::InvalidValue.into_result_code());
        }
    }

    mod str_from_utf8 {
        use super::*;

        #[test]
        fn should_return_a_str_from_utf8_bytes() {
            let result = str_from_utf8("test".as_bytes()).unwrap();
            assert_eq!(result, "test")
        }

        #[test]
        fn should_return_an_error_if_the_bytes_are_not_a_valid_utf8_string() {
            let bytes: [u8; 4] = [0x74, 0x65, 0x73, 0x9f];
            let result = str_from_utf8(&bytes).unwrap_err();
            assert_eq!(result, GenericResultCode::InvalidString.into_result_code())
        }
    }
}
