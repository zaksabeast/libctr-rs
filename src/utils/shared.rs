use crate::res::{CtrResult, GenericResultCode};
use base64::decode;

pub fn base64_encode<T: AsRef<[u8]>>(input: T) -> String {
    // This is super inefficient, but there's not really a
    // stellar base64 encoder available.
    // An option _might_ be too make one in the future?
    base64::encode_config(input, base64::URL_SAFE)
        .replace('=', "*")
        .replace('-', ".")
        .replace('_', "-")
}

pub fn base64_decode(base64: &str) -> CtrResult<Vec<u8>> {
    decode(base64.replace('*', "=")).map_err(|_| GenericResultCode::InvalidValue.into())
}

pub fn copy_into_slice<T: Copy>(src: &[T], dst: &mut [T]) -> CtrResult {
    let src_len = src.len();

    if src_len > dst.len() {
        return Err(GenericResultCode::InvalidSize.into());
    }

    dst[..src_len].copy_from_slice(src);

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    mod copy_into_slice {
        use super::*;

        #[test]
        fn should_copy_all_of_src_if_the_src_length_is_smaller_than_the_dst_length() {
            let src: [u8; 5] = [1; 5];
            let mut dst: [u8; 10] = [0; 10];

            copy_into_slice(&src, &mut dst).unwrap();

            assert_eq!(dst, [1, 1, 1, 1, 1, 0, 0, 0, 0, 0]);
        }

        #[test]
        fn should_copy_all_of_src_if_the_src_length_is_equal_to_the_dst_length() {
            let src: [u8; 5] = [1; 5];
            let mut dst: [u8; 5] = [0; 5];

            copy_into_slice(&src, &mut dst).unwrap();

            assert_eq!(dst, [1, 1, 1, 1, 1]);
        }

        #[test]
        fn should_return_an_error_if_the_src_length_is_greater_than_the_dst_length() {
            let src: [u8; 10] = [1; 10];
            let mut dst: [u8; 5] = [0; 5];

            let result = copy_into_slice(&src, &mut dst).unwrap_err();

            assert_eq!(dst, [0, 0, 0, 0, 0]);
            assert_eq!(result, GenericResultCode::InvalidSize.into_result_code());
        }
    }
}
