#[cfg(not(test))]
use crate::frd::result::FrdErrorCode;
use alloc::string::String;
#[cfg(not(test))]
use core::convert::TryInto;
use ctr::frd::{FriendProfile, GameKey, Mii};
#[cfg(not(test))]
use ctr::{result::CtrResult, utils::convert::bytes_to_utf16le_string};

pub struct MyData {
    pub my_nc_principal_id: u32,
    pub changed_bit_flags: u32,
    pub is_public_mode: bool,
    pub is_show_game_mode: bool,
    pub is_show_played_game: bool,
    pub my_favorite_game: GameKey,
    pub personal_comment: String,
    pub profile: FriendProfile,
    pub mac_address: String,
    pub console_serial_number: String,
    pub screen_name: String,
    pub mii: Mii,
}

#[cfg(not(test))]
impl MyData {
    // This explicitly mentions the endianness instead of From<[u8; 288]>
    pub fn try_from_le_bytes(raw_data: [u8; 288]) -> CtrResult<Self> {
        let header_bytes = raw_data[..8].try_into().unwrap();

        if u64::from_le_bytes(header_bytes) != 0x20101021444d5046 {
            return Err(FrdErrorCode::InvalidFriendListOrMyDataSaveFile.into());
        }

        let my_nc_principal_id_bytes = raw_data[16..20].try_into().unwrap();
        let changed_bit_flags_bytes = raw_data[24..28].try_into().unwrap();

        let title_id_bytes = raw_data[32..40].try_into().unwrap();
        let title_version_bytes = raw_data[40..44].try_into().unwrap();
        let game_key_unk_bytes = raw_data[44..48].try_into().unwrap();

        Ok(Self {
            my_nc_principal_id: u32::from_le_bytes(my_nc_principal_id_bytes),
            changed_bit_flags: u32::from_le_bytes(changed_bit_flags_bytes),
            is_public_mode: raw_data[28] != 0,
            is_show_game_mode: raw_data[29] != 0,
            is_show_played_game: raw_data[30] != 0,
            my_favorite_game: GameKey {
                title_id: u64::from_le_bytes(title_id_bytes),
                version: u32::from_le_bytes(title_version_bytes),
                unk: u32::from_le_bytes(game_key_unk_bytes),
            },
            personal_comment: bytes_to_utf16le_string(&raw_data[48..82])?,
            profile: FriendProfile {
                region: raw_data[88],
                country: raw_data[89],
                area: raw_data[90],
                language: raw_data[91],
                platform: raw_data[92],
                padding: raw_data[93..96].try_into().unwrap(),
            },
            mac_address: bytes_to_utf16le_string(&raw_data[104..130])?,
            console_serial_number: bytes_to_utf16le_string(&raw_data[130..162])?,
            screen_name: bytes_to_utf16le_string(&raw_data[162..184])?,
            mii: Mii::new(raw_data[187..283].try_into().unwrap()),
        })
    }
}
