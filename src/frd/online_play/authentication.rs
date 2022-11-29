use super::{
    base_request::create_game_server_request,
    utils::{parse_address, parse_datetime_from_base64, parse_num_from_base64},
};
use crate::frd::context::FriendServiceContext;
use alloc::str;
use core::str::FromStr;
use ctr::{
    http::HttpContext,
    result::CtrResult,
    time::SystemTimestamp,
    utils::{base64_decode, copy_into_slice},
};
use no_std_io::{EndianRead, EndianWrite};

#[derive(Debug, PartialEq, Eq, Clone, Copy, EndianRead, EndianWrite)]
#[repr(C)]
pub struct GameAuthenticationData {
    return_code: u32,
    http_status_code: u32,
    address: [u8; 32],
    port: u32,
    retry: u32,
    token: [u8; 256],
    timestamp: SystemTimestamp,
}

impl GameAuthenticationData {
    pub fn from_fetched_response(response: &str, http_status_code: u32) -> CtrResult<Self> {
        let mut game_auth_data = GameAuthenticationData {
            http_status_code,
            ..Default::default()
        };

        let field_delimeter = char::from_str("&").unwrap();
        let value_delimeter = char::from_str("=").unwrap();

        for field in response.split(field_delimeter) {
            let mut split_field = field.split(value_delimeter);
            let key = split_field.next();
            let value = split_field.next();

            match (key, value) {
                (Some("locator"), Some(inner_value)) => {
                    let decoded_value = base64_decode(inner_value)?;
                    let decoded_str = str::from_utf8(&decoded_value)?;
                    let (address, port) = parse_address(decoded_str)?;

                    copy_into_slice(address.as_bytes(), &mut game_auth_data.address)?;
                    game_auth_data.port = port;
                }
                (Some("retry"), Some(inner_value)) => {
                    game_auth_data.retry = parse_num_from_base64(inner_value)?;
                }
                (Some("returncd"), Some(inner_value)) => {
                    game_auth_data.return_code = parse_num_from_base64(inner_value)?;
                }
                (Some("token"), Some(inner_value)) => {
                    copy_into_slice(inner_value.as_bytes(), &mut game_auth_data.token)?;
                }
                (Some("datetime"), Some(inner_value)) => {
                    game_auth_data.timestamp = parse_datetime_from_base64(inner_value)?;
                }
                _ => {}
            }
        }

        Ok(game_auth_data)
    }
}

impl Default for GameAuthenticationData {
    fn default() -> Self {
        Self {
            return_code: 0,
            http_status_code: 0,
            address: [0; 32],
            port: 0,
            retry: 0,
            token: [0; 256],
            timestamp: SystemTimestamp::new(0),
        }
    }
}

pub fn create_game_login_request(
    context: &FriendServiceContext,
    requesting_process_id: u32,
    requesting_game_id: u32,
    sdk_version_low: u8,
    sdk_version_high: u8,
    ingamesn: &str,
) -> CtrResult<HttpContext> {
    let request = create_game_server_request(
        context,
        requesting_process_id,
        requesting_game_id,
        sdk_version_low,
        sdk_version_high,
    )?;
    request.add_post_base64_field("action", "LOGIN")?;
    request.add_post_base64_field("ingamesn", ingamesn)?;

    Ok(request)
}

#[cfg(test)]
mod test {
    use super::*;

    mod game_authentication_data {
        use super::*;
        use alloc::vec;
        use ctr::time::FormattedTimestamp;
        use no_std_io::Writer;

        #[test]
        fn should_parse_an_auth_response() {
            let auth_response = "locator=MTI3LjAuMC4xOjcwMDA*&retry=MA**&returncd=MDAx&token=AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8gISIjJCUmJygpKissLS4vMDE*.AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8gISIjJCUmJygpKissLS4vMDE*&datetime=MjAyMTAxMDIwMzA0MDU*";
            let parsed_response = GameAuthenticationData::from_fetched_response(auth_response, 200)
                .expect("Should have parsed the auth response");

            let address_bytes = "127.0.0.1".as_bytes();
            let mut address = [0; 32];
            address[..address_bytes.len()].clone_from_slice(address_bytes);

            let token_bytes = "AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8gISIjJCUmJygpKissLS4vMDE*.AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8gISIjJCUmJygpKissLS4vMDE*".as_bytes();
            let mut token = [0; 0x100];
            token[..token_bytes.len()].clone_from_slice(token_bytes);

            let expected_result = GameAuthenticationData {
                address,
                http_status_code: 200,
                port: 7000,
                retry: 0,
                return_code: 1,
                token,
                timestamp: FormattedTimestamp::new(2021, 1, 2, 3, 4, 5).into(),
            };

            assert_eq!(parsed_response, expected_result);

            let mut bytes = vec![];
            bytes.checked_write_le(0, &parsed_response);
            assert_eq!(
                bytes,
                [
                    0x01, 0x00, 0x00, 0x00, 0xc8, 0x00, 0x00, 0x00, 0x31, 0x32, 0x37, 0x2e, 0x30,
                    0x2e, 0x30, 0x2e, 0x31, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x58, 0x1b, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x41, 0x41, 0x45, 0x43,
                    0x41, 0x77, 0x51, 0x46, 0x42, 0x67, 0x63, 0x49, 0x43, 0x51, 0x6f, 0x4c, 0x44,
                    0x41, 0x30, 0x4f, 0x44, 0x78, 0x41, 0x52, 0x45, 0x68, 0x4d, 0x55, 0x46, 0x52,
                    0x59, 0x58, 0x47, 0x42, 0x6b, 0x61, 0x47, 0x78, 0x77, 0x64, 0x48, 0x68, 0x38,
                    0x67, 0x49, 0x53, 0x49, 0x6a, 0x4a, 0x43, 0x55, 0x6d, 0x4a, 0x79, 0x67, 0x70,
                    0x4b, 0x69, 0x73, 0x73, 0x4c, 0x53, 0x34, 0x76, 0x4d, 0x44, 0x45, 0x2a, 0x2e,
                    0x41, 0x41, 0x45, 0x43, 0x41, 0x77, 0x51, 0x46, 0x42, 0x67, 0x63, 0x49, 0x43,
                    0x51, 0x6f, 0x4c, 0x44, 0x41, 0x30, 0x4f, 0x44, 0x78, 0x41, 0x52, 0x45, 0x68,
                    0x4d, 0x55, 0x46, 0x52, 0x59, 0x58, 0x47, 0x42, 0x6b, 0x61, 0x47, 0x78, 0x77,
                    0x64, 0x48, 0x68, 0x38, 0x67, 0x49, 0x53, 0x49, 0x6a, 0x4a, 0x43, 0x55, 0x6d,
                    0x4a, 0x79, 0x67, 0x70, 0x4b, 0x69, 0x73, 0x73, 0x4c, 0x53, 0x34, 0x76, 0x4d,
                    0x44, 0x45, 0x2a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x88, 0xa8, 0x3d, 0x56, 0x9a, 0x00, 0x00, 0x00
                ]
            )
        }

        #[test]
        fn should_default_to_all_zeros() {
            let game_auth_data = GameAuthenticationData::default();
            let mut game_auth_bytes = vec![];
            game_auth_bytes.checked_write_le(0, &game_auth_data);
            assert_eq!(game_auth_bytes, [0; 312])
        }
    }
}
