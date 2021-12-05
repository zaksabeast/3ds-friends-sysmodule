use super::{
    base_request::create_game_server_request,
    utils::{parse_datetime_from_base64, parse_num_from_base64},
};
use crate::frd::context::FriendServiceContext;
use core::{str, str::FromStr};
use ctr::{
    http::HttpContext,
    result::CtrResult,
    time::SystemTimestamp,
    utils::{base64_decode, copy_into_slice},
};
use safe_transmute::TriviallyTransmutable;

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(C)]
pub struct ServiceLocateData {
    pub return_code: u32,
    pub http_status_code: u32,
    pub svc_host: [u8; 128],
    pub token: [u8; 256],
    pub status_data: [u8; 8],
    pub timestamp: SystemTimestamp,
}

// This is safe because all fields in the struct can function with any value.
// At some point it may be worth having a validator to ensure a valid value
// is sent to another process.
unsafe impl TriviallyTransmutable for ServiceLocateData {}

impl ServiceLocateData {
    pub fn from_fetched_response(response: &str, http_status_code: u32) -> CtrResult<Self> {
        let mut service_locate_data = ServiceLocateData {
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
                (Some("returncd"), Some(inner_value)) => {
                    service_locate_data.return_code = parse_num_from_base64(inner_value)?;
                }
                (Some("servicetoken"), Some(inner_value)) => {
                    copy_into_slice(inner_value.as_bytes(), &mut service_locate_data.token)?;
                }
                (Some("statusdata"), Some(inner_value)) => {
                    let decoded_value = base64_decode(inner_value)?;
                    copy_into_slice(&decoded_value, &mut service_locate_data.status_data)?;
                }
                (Some("svchost"), Some(inner_value)) => {
                    let decoded_value = base64_decode(inner_value)?;
                    copy_into_slice(&decoded_value, &mut service_locate_data.svc_host)?;
                }
                (Some("datetime"), Some(inner_value)) => {
                    service_locate_data.timestamp = parse_datetime_from_base64(inner_value)?;
                }
                _ => {}
            }
        }

        Ok(service_locate_data)
    }
}

impl Default for ServiceLocateData {
    fn default() -> Self {
        Self {
            return_code: 0,
            http_status_code: 0,
            svc_host: [0; 128],
            token: [0; 256],
            status_data: [0; 8],
            timestamp: SystemTimestamp::new(0),
        }
    }
}

#[cfg_attr(test, mocktopus::macros::mockable)]
pub fn create_game_service_locate_request(
    context: &FriendServiceContext,
    requesting_process_id: u32,
    requesting_game_id: u32,
    sdk_version_low: u8,
    sdk_version_high: u8,
    key_hash: &str,
    svc: &str,
) -> CtrResult<HttpContext> {
    let request = create_game_server_request(
        context,
        requesting_process_id,
        requesting_game_id,
        sdk_version_low,
        sdk_version_high,
    )?;
    request.add_post_base64_field("action", "SVCLOC")?;
    request.add_post_base64_field("keyhash", key_hash)?;
    request.add_post_base64_field("svc", svc)?;

    Ok(request)
}

#[cfg(test)]
mod test {
    use super::*;

    mod service_locate_data {
        use super::*;
        use ctr::time::FormattedTimestamp;
        use safe_transmute::transmute_one_to_bytes;

        #[test]
        fn should_parse_a_fetched_response() {
            let fetched_response = "retry=MA**&returncd=MDA3&servicetoken=AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8gISIjJCUmJygpKissLS4vMDE*&statusdata=WQ**&svchost=bi9h&datetime=MjAyMTAxMDIwMzA0MDU*";
            let parsed_response = ServiceLocateData::from_fetched_response(fetched_response, 200)
                .expect("Should have parsed the response");

            let status_data_bytes = "Y".as_bytes();
            let mut status_data = [0; 8];
            status_data[..status_data_bytes.len()].clone_from_slice(status_data_bytes);

            let svc_host_bytes = "n/a".as_bytes();
            let mut svc_host = [0; 128];
            svc_host[..svc_host_bytes.len()].clone_from_slice(svc_host_bytes);

            let token_bytes =
                "AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8gISIjJCUmJygpKissLS4vMDE*".as_bytes();
            let mut token = [0; 0x100];
            token[..token_bytes.len()].clone_from_slice(token_bytes);

            let expected_result = ServiceLocateData {
                status_data,
                http_status_code: 200,
                return_code: 7,
                token,
                svc_host,
                timestamp: FormattedTimestamp::new(2021, 1, 2, 3, 4, 5).into(),
            };

            assert_eq!(parsed_response, expected_result);
            assert_eq!(
                transmute_one_to_bytes(&parsed_response),
                [
                    0x07, 0x00, 0x00, 0x00, 0xc8, 0x00, 0x00, 0x00, 0x6e, 0x2f, 0x61, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x41, 0x41, 0x45, 0x43, 0x41, 0x77, 0x51,
                    0x46, 0x42, 0x67, 0x63, 0x49, 0x43, 0x51, 0x6f, 0x4c, 0x44, 0x41, 0x30, 0x4f,
                    0x44, 0x78, 0x41, 0x52, 0x45, 0x68, 0x4d, 0x55, 0x46, 0x52, 0x59, 0x58, 0x47,
                    0x42, 0x6b, 0x61, 0x47, 0x78, 0x77, 0x64, 0x48, 0x68, 0x38, 0x67, 0x49, 0x53,
                    0x49, 0x6a, 0x4a, 0x43, 0x55, 0x6d, 0x4a, 0x79, 0x67, 0x70, 0x4b, 0x69, 0x73,
                    0x73, 0x4c, 0x53, 0x34, 0x76, 0x4d, 0x44, 0x45, 0x2a, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x59, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x88, 0xa8, 0x3d,
                    0x56, 0x9a, 0x00, 0x00, 0x00
                ]
            )
        }

        #[test]
        fn should_default_to_all_zeros() {
            let game_auth_data = ServiceLocateData::default();
            let game_auth_bytes = transmute_one_to_bytes(&game_auth_data);

            let expected_result: [u8; 408] = [0; 408];
            assert_eq!(game_auth_bytes, expected_result)
        }
    }

    mod create_game_service_locate_request {
        use super::*;
        use ctr::{http::RequestMethod, utils::base64_encode};
        use mocktopus::mocking::{MockResult, Mockable};

        #[test]
        fn should_create_a_game_server_request_and_add_the_key_hash_and_svc_as_fields() {
            create_game_server_request.mock_safe(
                |_context,
                 _requesting_process_id,
                 _requesting_game_id,
                 _sdk_version_low,
                 _sdk_version_high| {
                    MockResult::Return(HttpContext::new("", RequestMethod::Post))
                },
            );

            let context = FriendServiceContext::new().unwrap();
            let request = create_game_service_locate_request(
                &context,
                1234,
                5678,
                20,
                21,
                "key_hash-test",
                "svc-test",
            )
            .expect("Login request should have been created!")
            .mock;
            let post_body_fields = &request.borrow().post_body_fields;

            assert_eq!(
                post_body_fields.get("keyhash"),
                Some(&base64_encode("key_hash-test"))
            );
            assert_eq!(
                post_body_fields.get("svc"),
                Some(&base64_encode("svc-test"))
            );
        }
    }
}