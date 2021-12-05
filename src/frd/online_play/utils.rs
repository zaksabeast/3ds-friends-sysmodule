use alloc::{str, vec::Vec};
use core::str::FromStr;
use ctr::{
    result::{CtrResult, GenericResultCode},
    time::{FormattedTimestamp, SystemTimestamp},
    utils::{
        base64_decode,
        parse::{parse_num, str_from_utf8},
    },
};

pub fn parse_address(full_address: &str) -> CtrResult<(&str, u32)> {
    let colon = char::from_str(":").unwrap();
    let mut split_address = full_address.split(colon);
    let address = split_address.next();
    let port = split_address.next();

    match (address, port) {
        (Some(address), Some(port)) => Ok((address, parse_num(port)?)),
        _ => Err(GenericResultCode::InvalidValue.into()),
    }
}

pub fn parse_datetime(datetime: &str) -> CtrResult<SystemTimestamp> {
    let time_slices = datetime
        .as_bytes()
        .chunks(2)
        .map(str::from_utf8)
        .collect::<Result<Vec<&str>, _>>()
        .map_err(|_| GenericResultCode::InvalidValue)?;

    if time_slices.len() != 7 {
        return Err(GenericResultCode::InvalidValue.into());
    }

    let year: u16 = parse_num(time_slices[1])?;
    let month: u16 = parse_num(time_slices[2])?;
    let date: u16 = parse_num(time_slices[3])?;
    let hours: u16 = parse_num(time_slices[4])?;
    let minutes: u16 = parse_num(time_slices[5])?;
    let seconds: u16 = parse_num(time_slices[6])?;

    let parsed_timestamp =
        FormattedTimestamp::new(year + 2000, month, date, hours, minutes, seconds);

    Ok(parsed_timestamp.into())
}

pub fn parse_num_from_base64<T: FromStr>(base64: &str) -> CtrResult<T> {
    let decoded_bytes = base64_decode(base64)?;
    let decoded_str = str_from_utf8(&decoded_bytes)?;
    parse_num(decoded_str)
}

pub fn parse_datetime_from_base64(base64: &str) -> CtrResult<SystemTimestamp> {
    let decoded_bytes = base64_decode(base64)?;
    let decoded_str = str_from_utf8(&decoded_bytes)?;
    parse_datetime(decoded_str)
}
