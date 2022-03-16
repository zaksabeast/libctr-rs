use crate::{
    result::{CtrResult, GenericResultCode},
    safe_transmute::transmute_many_pedantic,
};
use alloc::string::String;
use core::convert::TryInto;

pub fn try_usize_into_u32(size: usize) -> Result<u32, GenericResultCode> {
    let word: u32 = size
        .try_into()
        .map_err(|_| GenericResultCode::InvalidSize)?;
    Ok(word)
}

pub fn bytes_to_utf16le_string(bytes: &[u8]) -> CtrResult<String> {
    let shorts = transmute_many_pedantic(bytes)?;
    let zero_index = shorts
        .iter()
        .position(|num| *num == 0)
        .unwrap_or(shorts.len());
    String::from_utf16(&shorts[..zero_index]).map_err(|_| GenericResultCode::InvalidString.into())
}

pub fn u8_slice_to_u32(bytes: &[u8]) -> u32 {
    let mut arr: [u8; 4] = [0; 4];

    for (index, byte) in bytes.iter().enumerate().take(4) {
        arr[index] = *byte;
    }

    u32::from_ne_bytes(arr)
}

#[cfg(test)]
mod test {
    use super::*;

    mod bytes_to_utf16le_string {
        use super::*;

        #[test]
        fn should_convert_bytes_to_utf16le_string() {
            let bytes = [0x54, 0x00, 0x65, 0x00, 0x73, 0x00, 0x74, 0x00];
            let result = bytes_to_utf16le_string(&bytes)
                .expect("Expected string. Bytes were not converted correctly");

            assert_eq!(result, "Test");
        }

        #[test]
        fn should_not_include_null_terminators() {
            let bytes = [0x54, 0x00, 0x65, 0x00, 0x73, 0x00, 0x74, 0x00, 0x00, 0x00];
            let result = bytes_to_utf16le_string(&bytes)
                .expect("Expected string. Bytes were not converted correctly");

            assert_eq!(result, "Test");
        }

        #[test]
        fn should_error_if_unaligned_bytes() {
            let bytes = [0x54, 0x00, 0x65, 0x00, 0x73, 0x00, 0x74];
            let result =
                bytes_to_utf16le_string(&bytes).expect_err("Expected error for unaligned bytes");

            assert_eq!(result, GenericResultCode::AlignmentError.into_result_code());
        }

        #[test]
        fn should_error_if_invalid_utf16_bytes() {
            let bytes = [0x54, 0x00, 0x65, 0x00, 0x00, 0xd8, 0x74, 0x00];
            let result = bytes_to_utf16le_string(&bytes)
                .expect_err("Expected error for invalid utf16 bytes");

            assert_eq!(result, GenericResultCode::InvalidString.into_result_code());
        }
    }

    mod u8_slice_to_u32 {
        use super::*;

        #[test]
        fn should_return_a_u32_from_a_u8_slice() {
            let bytes = [0xaa, 0xbb, 0xcc, 0xdd];
            let result = u8_slice_to_u32(&bytes);

            assert_eq!(result, 0xddccbbaa);
        }

        #[test]
        fn should_return_a_u32_from_an_unaligned_u8_slice() {
            let bytes = [0xaa, 0xbb, 0xcc];
            let result = u8_slice_to_u32(&bytes);

            assert_eq!(result, 0xccbbaa);
        }

        #[test]
        fn should_return_a_u32_from_an_empty_u8_slice() {
            let bytes = [];
            let result = u8_slice_to_u32(&bytes);

            assert_eq!(result, 0);
        }
    }
}
