use crate::frd::result::FrdErrorCode;
use alloc::{format, string::String};
use core::convert::TryInto;
use ctr::{result::CtrResult, utils::convert::bytes_to_utf16le_string};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum NascEnvironment {
    Prod = 0,
    Test = 1,
    Dev = 2,
}

impl From<u8> for NascEnvironment {
    fn from(byte: u8) -> Self {
        match byte {
            0 => Self::Prod,
            1 => Self::Test,
            2 => Self::Dev,
            _ => Self::Prod,
        }
    }
}

pub struct AccountConfig {
    pub local_account_id: u32,
    pub principal_id: u32,
    pub local_friend_code: u64,
    pub nex_password: String,
    pub principal_id_hmac: String,
    pub nasc_environment: NascEnvironment,
    pub server_type_1: u8,
    pub server_type_2: u8,
}

impl AccountConfig {
    pub fn try_from_le_bytes(raw_data: [u8; 88]) -> CtrResult<Self> {
        let header_bytes = raw_data[..8].try_into().unwrap();

        if u64::from_le_bytes(header_bytes) != 0x2010102143415046 {
            return Err(FrdErrorCode::InvalidAccountSaveFile.into());
        }

        let local_account_id_bytes = raw_data[16..20].try_into().unwrap();
        let principal_id_bytes = raw_data[20..24].try_into().unwrap();
        let local_friend_code_bytes = raw_data[24..32].try_into().unwrap();

        Ok(Self {
            local_account_id: u32::from_le_bytes(local_account_id_bytes),
            principal_id: u32::from_le_bytes(principal_id_bytes),
            local_friend_code: u64::from_le_bytes(local_friend_code_bytes),
            nex_password: bytes_to_utf16le_string(&raw_data[32..64])?,
            principal_id_hmac: bytes_to_utf16le_string(&raw_data[66..84])?,
            nasc_environment: raw_data[84].into(),
            server_type_1: raw_data[85],
            server_type_2: raw_data[86],
        })
    }

    pub fn get_server_type_string(&self) -> String {
        let server_type_1_letter = match self.server_type_1 {
            0 => "L",
            1 => "C",
            2 => "S",
            3 => "D",
            4 => "I",
            5 => "T",
            7 => "J",
            8 => "X",
            9 => "A",
            10 => "B",
            11 => "C",
            12 => "D",
            13 => "E",
            14 => "F",
            15 => "G",
            16 => "H",
            17 => "I",
            18 => "J",
            19 => "K",
            20 => "L",
            21 => "M",
            22 => "N",
            23 => "O",
            24 => "P",
            25 => "Q",
            _ => "U",
        };

        format!("{}{}", server_type_1_letter, self.server_type_2)
    }
}
