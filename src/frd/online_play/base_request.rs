use crate::frd::context::FriendServiceContext;
use alloc::{format, str, vec::Vec};
use ctr::{
    ac::{acu_get_current_ap_info, acu_get_wifi_status},
    cfg::{get_console_username, get_local_friend_code_seed_data},
    fs,
    fs::MediaType,
    http::{DefaultRootCert, HttpContext, RequestMethod},
    os::get_time,
    ps::get_rom_id,
    result::{CtrResult, GenericResultCode},
    time::SystemTimestamp,
    utils::cstring::parse_null_terminated_str,
};
use safe_transmute::transmute_to_bytes;

#[cfg_attr(test, mocktopus::macros::mockable)]
pub fn create_game_server_request(
    context: &FriendServiceContext,
    requesting_process_id: u32,
    requesting_game_id: u32,
    sdk_version_low: u8,
    sdk_version_high: u8,
) -> CtrResult<HttpContext> {
    let url = "https://nasc.nintendowifi.net/ac";
    let request = HttpContext::new(url, RequestMethod::Post)?;

    request.add_default_cert(DefaultRootCert::NintendoCa)?;
    request.add_default_cert(DefaultRootCert::NintendoCaG2)?;
    request.add_default_cert(DefaultRootCert::NintendoCaG3)?;
    request.set_client_cert_default()?;

    request.add_header("X-GameId", &format!("{:08X}", requesting_game_id))?;
    // The official sysmodule effectively does `format!("CTR FPD/{:04X}", get_value())`,
    // however `get_value` is set to always return 0xF.
    request.add_header("User-Agent", "CTR FPD/000F")?;
    // It looks like the http sysmodule adds this automatically,
    // yet the frd sysmodule still adds this.  The http sysmodule doesn't
    // recognize it has added the same header twice, so this appears twice
    // in requests.
    // I'm going to keep this for now to mimic online play as best as possible,
    // but this should be removed once official servers are down.
    request.add_header("Content-Type", "application/x-www-form-urlencoded")?;

    let program_info = fs::user::get_program_launch_info(requesting_process_id)?;
    let product_info = fs::user::get_product_info(requesting_process_id)?;

    request.add_post_base64_field("gameid", &format!("{:08X}", requesting_game_id))?;
    request.add_post_base64_field(
        "sdkver",
        &format!("{:03}{:03}", sdk_version_low, sdk_version_high),
    )?;
    request.add_post_base64_field("titleid", &format!("{:016X}", program_info.program_id))?;
    // The friends list app always uses "----", but it's the only thing
    // Since the friends online play is not being added, we don't have to worry about it
    let product_code = parse_null_terminated_str(&product_info.product_code[6..10]);
    request.add_post_base64_field("gamecd", product_code)?;
    request.add_post_base64_field("gamever", &format!("{:04X}", product_info.remaster_version))?;
    request.add_post_base64_field("mediatype", &format!("{}", program_info.media_type as u8))?;

    if program_info.media_type == MediaType::GameCard {
        let rom_id = get_rom_id(requesting_process_id)?;
        request.add_post_base64_field("romid", rom_id.get_inner())?;
    }

    let company_code =
        str::from_utf8(&product_info.company_code).map_err(|_| GenericResultCode::InvalidString)?;
    request.add_post_base64_field("makercd", company_code)?;
    request.add_post_base64_field("unitcd", "2")?;
    request.add_post_base64_field("macadr", &context.my_data.mac_address)?;

    let ap_info = acu_get_current_ap_info()?;
    request.add_post_base64_field("bssid", &ap_info.get_formatted_bssid())?;

    // This normally uses ACU_GetWifiStatus, ACU_GetNZoneApNumService, and ACU_GetConnectingHotspotSubset,
    // but NZone is down and most people should always have the same data here, so we'll skip the extra logic for now.
    let wifi_status = acu_get_wifi_status()?;
    request.add_post_base64_field("apinfo", &format!("{:02}:0000000000", wifi_status))?;

    let local_friend_code_seed = get_local_friend_code_seed_data()?;
    request.add_post_base64_field("fcdcert", &local_friend_code_seed)?;

    let console_username = get_console_username()?.encode_utf16().collect::<Vec<u16>>();
    request.add_post_base64_field("devname", transmute_to_bytes(&console_username))?;

    // Has special formatting
    request.add_post_base64_field(
        "servertype",
        context.account_config.get_server_type_string(),
    )?;

    // This looks to be hardcoded to '000F', but I'm curious if that's the case for all models/fw versions
    request.add_post_base64_field("fpdver", "000F")?;

    let current_time = SystemTimestamp::new(get_time());
    let current_year_month_date = current_time.get_year_month_date();
    request.add_post_base64_field(
        "devtime",
        &format!(
            "{:02}{:02}{:02}{:02}{:02}{:02}",
            current_year_month_date.year % 100,
            current_year_month_date.month,
            current_year_month_date.date,
            current_time.get_hours(),
            current_time.get_minutes(),
            current_time.get_seconds()
        ),
    )?;

    request.add_post_base64_field("lang", &format!("{:02X}", context.my_data.profile.language))?;
    request.add_post_base64_field("region", &format!("{:02X}", context.my_data.profile.region))?;
    request.add_post_base64_field("csnum", &context.my_data.console_serial_number)?;

    // Interestingly at this point, the official implementation sends the user's
    // password as a post body field if the user's principal_id is 0.
    // We're not going to do that.

    request.add_post_base64_field("uidhmac", &context.account_config.principal_id_hmac)?;
    request.add_post_base64_field(
        "userid",
        &format!("{}", context.account_config.principal_id),
    )?;

    Ok(request)
}

#[cfg(test)]
mod test {
    use super::*;

    mod create_game_server_request {
        use super::*;
        use crate::frd::save::account::AccountConfig;
        use alloc::{string::ToString, vec, vec::Vec};
        use core::convert::TryInto;
        use ctr::{
            ac::ApInfo,
            fs::{ProductInfo, ProgramInfo},
            http::RequestMethod,
            ps::RomId,
            time::FormattedTimestamp,
            utils::base64_encode,
        };
        use mocktopus::mocking::{MockResult, Mockable};

        fn setup_mocks(local_friend_code_seed: [u8; 0x110]) {
            get_console_username.mock_safe(|| {
                let username = "TestUser".to_string();
                MockResult::Return(Ok(username))
            });
            get_local_friend_code_seed_data
                .mock_safe(move || MockResult::Return(Ok(local_friend_code_seed)));
            fs::user::get_product_info.mock_safe(|_| {
                let mut product_code: [u8; 16] = [0; 16];
                product_code[..10].copy_from_slice("CTR-U-ABCD".as_bytes());

                MockResult::Return(Ok(ProductInfo {
                    product_code,
                    remaster_version: 12,
                    company_code: "03".as_bytes().try_into().unwrap(),
                }))
            });
            fs::user::get_program_launch_info.mock_safe(|_| {
                MockResult::Return(Ok(ProgramInfo {
                    program_id: 0x0000029580019285,
                    media_type: MediaType::Nand,
                    ..Default::default()
                }))
            });
            let system_time: SystemTimestamp = FormattedTimestamp::new(2021, 1, 1, 12, 0, 0).into();
            get_time.mock_safe(move || MockResult::Return(system_time.get_epoch()));
            AccountConfig::get_server_type_string
                .mock_safe(|_| MockResult::Return("U".to_string()));
            acu_get_wifi_status.mock_safe(|| MockResult::Return(Ok(1)));
            acu_get_current_ap_info.mock_safe(|| {
                let ap_info: ApInfo = ApInfo {
                    bssid: [1, 2, 3, 4, 5, 6],
                    ..Default::default()
                };
                MockResult::Return(Ok(ap_info))
            })
        }

        #[test]
        fn should_return_an_authentication_request() {
            let local_friend_code_seed: [u8; 0x110] = (0..0x110)
                .map(|num| (num % 255) as u8)
                .collect::<Vec<u8>>()
                .try_into()
                .unwrap();

            setup_mocks(local_friend_code_seed.clone());

            let mut context = FriendServiceContext::new().unwrap();
            context.my_data.profile.language = 3;
            context.my_data.profile.region = 4;
            context.my_data.mac_address = "51627384".to_string();
            context.my_data.console_serial_number = "1234567890".to_string();
            context.account_config.principal_id_hmac = "0987654321".to_string();
            context.account_config.principal_id = 20;

            let request = create_game_server_request(&context, 1234, 5678, 20, 21)
                .expect("Auth request should have been created!")
                .mock;
            let headers = &request.borrow().headers;
            let post_body_fields = &request.borrow().post_body_fields;

            assert_eq!(request.borrow().method, RequestMethod::Post);
            assert_eq!(headers.get("X-GameId"), Some(&"0000162E".to_string()));
            assert_eq!(headers.get("User-Agent"), Some(&"CTR FPD/000F".to_string()));
            assert_eq!(
                headers.get("Content-Type"),
                Some(&"application/x-www-form-urlencoded".to_string())
            );
            assert_eq!(
                post_body_fields.get("gameid"),
                Some(&base64_encode("0000162E"))
            );
            assert_eq!(
                post_body_fields.get("sdkver"),
                Some(&base64_encode("020021"))
            );
            assert_eq!(
                post_body_fields.get("titleid"),
                Some(&base64_encode("0000029580019285"))
            );
            assert_eq!(post_body_fields.get("gamecd"), Some(&base64_encode("ABCD")));
            assert_eq!(
                post_body_fields.get("gamever"),
                Some(&base64_encode("000C"))
            );
            assert_eq!(post_body_fields.get("mediatype"), Some(&base64_encode("0")));
            assert_eq!(post_body_fields.get("makercd"), Some(&base64_encode("03")));
            assert_eq!(post_body_fields.get("unitcd"), Some(&base64_encode("2")));
            assert_eq!(
                post_body_fields.get("macadr"),
                Some(&base64_encode("51627384"))
            );

            assert_eq!(
                post_body_fields.get("bssid"),
                Some(&base64_encode("010203040506"))
            );

            assert_eq!(
                post_body_fields.get("apinfo"),
                Some(&base64_encode("01:0000000000"))
            );
            assert_eq!(
                post_body_fields.get("fcdcert"),
                Some(&base64_encode(local_friend_code_seed))
            );
            let username = "TestUser".encode_utf16().collect::<Vec<u16>>();
            assert_eq!(
                post_body_fields.get("devname"),
                Some(&base64_encode(transmute_to_bytes(&username)))
            );
            assert_eq!(
                post_body_fields.get("servertype"),
                Some(&base64_encode("U"))
            );
            assert_eq!(post_body_fields.get("fpdver"), Some(&base64_encode("000F")));
            assert_eq!(
                post_body_fields.get("devtime"),
                Some(&base64_encode("210101120000"))
            );
            assert_eq!(post_body_fields.get("lang"), Some(&base64_encode("03")));
            assert_eq!(post_body_fields.get("region"), Some(&base64_encode("04")));
            assert_eq!(
                post_body_fields.get("csnum"),
                Some(&base64_encode("1234567890"))
            );
            assert_eq!(
                post_body_fields.get("uidhmac"),
                Some(&base64_encode("0987654321"))
            );
            assert_eq!(post_body_fields.get("userid"), Some(&base64_encode("20")));
        }

        #[test]
        fn should_add_a_rom_id_post_body_field_if_the_program_media_type_is_a_game_card() {
            let local_friend_code_seed: [u8; 0x110] = [0; 0x110];
            setup_mocks(local_friend_code_seed);

            fs::user::get_program_launch_info.mock_safe(|_| {
                MockResult::Return(Ok(ProgramInfo {
                    program_id: 0x0000029580019285,
                    media_type: MediaType::GameCard,
                    ..Default::default()
                }))
            });

            let rom_id: [u8; 16] = [
                0x00, 0x00, 0x00, 0x92, 0xAC, 0x00, 0x00, 0x83, 0x02, 0x90, 0x04, 0x00, 0x20, 0xA0,
                0x00, 0x00,
            ];
            get_rom_id.mock_safe(move |_| MockResult::Return(Ok(RomId::new(rom_id))));

            let context = FriendServiceContext::new().unwrap();
            let request = create_game_server_request(&context, 1234, 5678, 20, 21)
                .expect("Auth request should have been created!")
                .mock;
            let post_body_fields = &request.borrow().post_body_fields;

            assert_eq!(post_body_fields.get("romid"), Some(&base64_encode(rom_id)));
        }

        #[test]
        fn should_not_add_a_rom_id_post_body_field_if_the_program_media_type_is_not_a_game_card() {
            let local_friend_code_seed: [u8; 0x110] = [0; 0x110];
            setup_mocks(local_friend_code_seed);

            fs::user::get_program_launch_info.mock_safe(|_| {
                MockResult::Return(Ok(ProgramInfo {
                    program_id: 0x0000029580019285,
                    media_type: MediaType::Nand,
                    ..Default::default()
                }))
            });

            let context = FriendServiceContext::new().unwrap();
            let request = create_game_server_request(&context, 1234, 5678, 20, 21)
                .expect("Auth request should have been created!")
                .mock;
            let post_body_fields = &request.borrow().post_body_fields;

            assert_eq!(post_body_fields.get("romid"), None);
        }

        #[test]
        fn should_add_certs_to_the_request() {
            let local_friend_code_seed: [u8; 0x110] = [0; 0x110];
            setup_mocks(local_friend_code_seed);

            let context = FriendServiceContext::new().unwrap();
            let request = create_game_server_request(&context, 1234, 5678, 20, 21)
                .expect("Auth request should have been created!")
                .mock;
            let request = &request.borrow();

            assert_eq!(
                request.certs,
                vec![
                    DefaultRootCert::NintendoCa,
                    DefaultRootCert::NintendoCaG2,
                    DefaultRootCert::NintendoCaG3,
                ]
            );
            assert!(request.has_client_cert)
        }
    }
}
