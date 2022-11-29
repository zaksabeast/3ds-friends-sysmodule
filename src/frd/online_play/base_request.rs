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
    result::CtrResult,
    time::SystemTimestamp,
    utils::cstring::parse_null_terminated_str,
};

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

    let company_code = str::from_utf8(&product_info.company_code)?;
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
    request.add_post_base64_field("fcdcert", local_friend_code_seed)?;

    let console_username = get_console_username()?
        .encode_utf16()
        .flat_map(|short| short.to_le_bytes())
        .collect::<Vec<u8>>();
    request.add_post_base64_field("devname", &console_username)?;

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
