use alloc::{str, vec::Vec};
use core::str::FromStr;
use ctr::{
    result::{error, CtrResult},
    time::{FormattedTimestamp, SystemTimestamp},
    utils::base64_decode,
};

pub fn parse_address(full_address: &str) -> CtrResult<(&str, u32)> {
    let colon = char::from_str(":").unwrap();
    let mut split_address = full_address.split(colon);
    let address = split_address.next();
    let port = split_address.next();

    match (address, port) {
        (Some(address), Some(port)) => Ok((address, port.parse()?)),
        _ => Err(error::invalid_value()),
    }
}

pub fn parse_datetime(datetime: &str) -> CtrResult<SystemTimestamp> {
    let time_slices = datetime
        .as_bytes()
        .chunks(2)
        .map(str::from_utf8)
        .collect::<Result<Vec<&str>, _>>()?;

    if time_slices.len() != 7 {
        return Err(error::invalid_value());
    }

    let year: u16 = time_slices[1].parse()?;
    let month: u16 = time_slices[2].parse()?;
    let date: u16 = time_slices[3].parse()?;
    let hours: u16 = time_slices[4].parse()?;
    let minutes: u16 = time_slices[5].parse()?;
    let seconds: u16 = time_slices[6].parse()?;

    let parsed_timestamp =
        FormattedTimestamp::new(year + 2000, month, date, hours, minutes, seconds);

    Ok(parsed_timestamp.into())
}

pub fn parse_num_from_base64<T: FromStr>(base64: &str) -> CtrResult<T> {
    let decoded_bytes = base64_decode(base64)?;
    let decoded_str = str::from_utf8(&decoded_bytes)?;
    decoded_str.parse().map_err(|_| error::invalid_value())
}

pub fn parse_datetime_from_base64(base64: &str) -> CtrResult<SystemTimestamp> {
    let decoded_bytes = base64_decode(base64)?;
    let decoded_str = str::from_utf8(&decoded_bytes)?;
    parse_datetime(decoded_str)
}
