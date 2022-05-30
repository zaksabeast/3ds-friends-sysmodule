use super::FriendServiceContext;
use crate::frd::{
    save::{
        account::AccountConfig,
        account::NascEnvironment,
        friend_list::{FriendEntry, MAX_FRIEND_COUNT},
        my_data::MyData,
    },
    wifi::WiFiConnectionStatus,
};
use alloc::{string::ToString, vec, vec::Vec};
use core::mem;
use ctr::{
    frd::{FriendKey, FriendProfile, GameKey, Mii},
    result::CtrResult,
};
use no_std_io::{EndianWrite, StreamContainer, StreamWriter};

impl FriendServiceContext {
    pub fn new() -> CtrResult<Self> {
        let ndm_wifi_event_handle = 0.into();

        let account_config = AccountConfig {
            local_account_id: 1,
            principal_id: 2,
            local_friend_code: 0xAAAAAAAABBBBBBBB,
            nex_password: "TestPassword!!!!".to_string(),
            principal_id_hmac: "11111111".to_string(),
            nasc_environment: NascEnvironment::Prod,
            server_type_1: 1,
            server_type_2: 2,
        };

        let my_data = MyData {
            my_nc_principal_id: 2,
            changed_bit_flags: 0,
            is_public_mode: false,
            is_show_game_mode: false,
            is_show_played_game: false,
            my_favorite_game: GameKey {
                title_id: 0xAAAAAAAABBBBBBBB,
                version: 1,
                unk: 0,
            },
            personal_comment: "TestMessage".to_string(),
            profile: FriendProfile {
                region: 1,
                country: 1,
                area: 1,
                language: 1,
                platform: 2,
                padding: [0; 3],
            },
            mac_address: "111111111111".to_string(),
            console_serial_number: "111111111111111".to_string(),
            screen_name: "TestUser".to_string(),
            mii: Mii::new([0; 96]),
        };

        let mut friend_list = Vec::with_capacity(MAX_FRIEND_COUNT);
        friend_list.push(FriendEntry {
            friend_key: FriendKey {
                principal_id: 1,
                padding: 0,
                local_friend_code: 0xCCCCCCCCDDDDDDDD,
            },
            favorite_game: GameKey {
                title_id: 0xAAAAAAAABBBBBBBB,
                version: 1,
                unk: 0,
            },
            comment: "TestMessage".into(),
            screen_name: "TestUser".into(),
            ..Default::default()
        });

        Ok(Self {
            ndm_wifi_event_handle,
            ndm_wifi_state: 0,
            wifi_connection_status: WiFiConnectionStatus::Idle,
            friend_list,
            counter: 0,
            account_config,
            my_data,
            my_online_activity: Default::default(),
            nat_properties: Default::default(),
            session_contexts: vec![],
            friend_key_list: [Default::default(); MAX_FRIEND_COUNT],
        })
    }

    pub fn get_friend_keys(&mut self) -> &[FriendKey] {
        for (index, friend) in self.friend_list.iter().enumerate() {
            self.friend_key_list[index] = friend.friend_key;
        }

        &self.friend_key_list[..self.friend_list.len()]
    }

    pub fn get_friend_by_friend_key(&self, friend_key: &FriendKey) -> Option<&FriendEntry> {
        self.friend_list
            .iter()
            .find(|friend_entry| friend_entry.friend_key == *friend_key)
    }

    pub fn get_session_static_buffer(&self, session_index: usize) -> &[u8] {
        &self.session_contexts[session_index].static_buffer
    }

    pub fn copy_into_session_static_buffer<T: EndianWrite + Sized>(
        &mut self,
        session_index: usize,
        data: &[T],
    ) -> &[u8] {
        let static_buffer = &mut self.session_contexts[session_index].static_buffer;
        static_buffer.clear();
        static_buffer.resize(data.len() * mem::size_of::<T>(), 0);
        let mut stream = StreamContainer::new(static_buffer.as_mut_slice());

        for datum in data.iter() {
            stream.checked_write_stream_le(datum);
        }

        stream.into_raw()
    }
}
