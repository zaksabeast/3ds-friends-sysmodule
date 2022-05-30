use ctr::{
    frd::{
        FriendComment, FriendInfo, FriendKey, FriendProfile, GameKey, Mii, ScreenName,
        SomeFriendThing, TrivialCharacterSet,
    },
    time::FormattedTimestamp,
};
use no_std_io::{EndianRead, EndianWrite};

pub const MAX_FRIEND_COUNT: usize = 100;

#[derive(Clone, Copy, Debug, PartialEq, Default, EndianRead, EndianWrite)]
#[repr(C)]
pub struct FriendEntry {
    pub friend_key: FriendKey,
    pub unk1: u32,
    pub friend_relationship: u8,
    pub friend_profile: FriendProfile,
    pub padding: [u8; 3],
    pub favorite_game: GameKey,
    pub comment: FriendComment,
    pub unk2: [u8; 6],
    pub timestamp1: FormattedTimestamp,
    pub timestamp2: FormattedTimestamp,
    pub last_online: FormattedTimestamp,
    pub mii: Mii,
    pub screen_name: ScreenName,
    pub unk3: u8,
    pub character_set: TrivialCharacterSet,
    pub timestamp3: FormattedTimestamp,
    pub timestamp1_2: FormattedTimestamp,
    pub timestamp2_2: FormattedTimestamp,
}

impl From<FriendEntry> for FriendInfo {
    fn from(friend_entry: FriendEntry) -> Self {
        Self {
            friend_key: friend_entry.friend_key,
            some_timestamp: Default::default(),
            friend_relationship: 3,
            unk1: [0, 0, 0],
            unk2: 0,
            unk3: SomeFriendThing {
                friend_profile: FriendProfile {
                    region: friend_entry.friend_profile.region,
                    country: friend_entry.friend_profile.country,
                    area: friend_entry.friend_profile.area,
                    language: friend_entry.friend_profile.language,
                    platform: friend_entry.friend_profile.platform,
                    padding: [0; 3],
                },
                favorite_game: friend_entry.favorite_game,
                unk2: 0,
                comment: friend_entry.comment,
                unk3: 0,
                last_online: friend_entry.last_online.into(),
            },
            screen_name: friend_entry.screen_name,
            character_set: friend_entry.character_set,
            unk4: 0,
            mii: friend_entry.mii,
        }
    }
}

const FRIEND_ATTRIBUTE: [u32; 6] = [0, 3, 0, 1, 1, 0];

impl FriendEntry {
    pub fn get_attribute(&self) -> u32 {
        if self.friend_relationship > 5 {
            return 3;
        }

        FRIEND_ATTRIBUTE[self.friend_relationship as usize]
    }
}
