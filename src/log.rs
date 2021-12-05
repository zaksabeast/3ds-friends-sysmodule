use ctr::Logger;
use lazy_static::lazy_static;

lazy_static! {
    static ref LOGGER: Logger = Logger::new("/frd-rs.txt");
}

pub fn debug(text: &str) {
    LOGGER.debug(text)
}
