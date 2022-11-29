use crate::frd::{
    online_play::{authentication::GameAuthenticationData, locate::ServiceLocateData},
    save::{
        account::AccountConfig,
        friend_list::{FriendEntry, MAX_FRIEND_COUNT},
        my_data::MyData,
    },
    wifi::WiFiConnectionStatus,
};
use alloc::{vec, vec::Vec};
use core::mem;
use ctr::{
    frd::{FriendKey, GameKey, NatProperties, NotificationEvent},
    fs::{ArchiveId, File, FsArchive, FsPath, OpenFlags},
    result::CtrResult,
    svc,
    svc::EventResetType,
    Handle,
};
use no_std_io::{EndianWrite, Reader, StreamContainer, StreamWriter};

#[derive(Default)]
pub struct OnlineActivity {
    pub playing_game: GameKey,
}

pub struct SessionContext {
    pub last_game_authentication_response: Option<GameAuthenticationData>,
    pub last_service_locator_response: Option<ServiceLocateData>,
    pub static_buffer: Vec<u8>,
    pub process_id: u32,
    pub client_sdk_version: u32,
    pub notification_mask: u32,
    pub server_time_interval: u64,
    pub client_event: Option<Handle>,
    // TODO: Add a mechanism that uses the notification_mask
    pub client_event_queue: Vec<NotificationEvent>,
}

impl SessionContext {
    pub fn new() -> Self {
        Self {
            last_game_authentication_response: None,
            last_service_locator_response: None,
            static_buffer: vec![],
            process_id: 0,
            client_sdk_version: 0,
            notification_mask: 0,
            server_time_interval: 0,
            client_event: None,
            client_event_queue: vec![],
        }
    }
}

/// Context needed for the FRD services.
pub struct FriendServiceContext {
    pub ndm_wifi_event_handle: Handle,
    pub ndm_wifi_state: u8,
    pub wifi_connection_status: WiFiConnectionStatus,
    pub counter: u32,
    pub account_config: AccountConfig,
    pub my_data: MyData,
    pub my_online_activity: OnlineActivity,
    pub nat_properties: NatProperties,
    pub friend_list: Vec<FriendEntry>,
    pub session_contexts: Vec<SessionContext>,
    // This needs to be an array so we can guarantee the pointer
    // to the underlying data never changes.
    // This is important for FrdUCommand::GetFriendKeyList.
    pub(super) friend_key_list: [FriendKey; MAX_FRIEND_COUNT],
}

impl FriendServiceContext {
    pub fn accept_session(&mut self) {
        let session_context = SessionContext::new();
        self.session_contexts.push(session_context);
    }

    pub fn close_session(&mut self, session_index: usize) {
        self.session_contexts.remove(session_index);
    }
}

fn get_my_account(archive: &FsArchive) -> CtrResult<AccountConfig> {
    let account_file: [u8; 88] = archive
        .open_file(&"/1/account".into(), OpenFlags::Read)?
        .read(0, 88)?
        .read_le(0)?;

    AccountConfig::try_from_le_bytes(account_file)
}

fn get_my_data(archive: &FsArchive) -> CtrResult<MyData> {
    let my_data_file: [u8; 288] = archive
        .open_file(&"/1/mydata".into(), OpenFlags::Read)?
        .read(0, 288)?
        .read_le(0)?;

    MyData::try_from_le_bytes(my_data_file)
}

fn read_friend_entry(friend_file: &File, index: u64) -> Option<FriendEntry> {
    friend_file
        .read((index * 0x100) + 16, 0x100)
        .ok()?
        .read_le(0)
        .ok()
}

fn read_friend_list(friend_list: &mut Vec<FriendEntry>, friend_file: &File) -> CtrResult<()> {
    for index in 0..MAX_FRIEND_COUNT {
        if let Some(friend_entry) = read_friend_entry(friend_file, index as u64) {
            friend_list.push(friend_entry);
        } else {
            break;
        }
    }

    Ok(())
}

impl FriendServiceContext {
    pub fn new() -> CtrResult<Self> {
        let ndm_wifi_event_handle = svc::create_event(EventResetType::OneShot)?;

        let save_archive_path = FsPath::new_binary([0, 0x10032]);
        let archive = FsArchive::new(ArchiveId::SystemSaveData, &save_archive_path)?;

        // TODO: Don't assume the user is using account 1
        let friend_list_path: FsPath = "/1/friendlist".into();
        let friend_file = archive.open_file(&friend_list_path, OpenFlags::Read)?;

        let mut friend_list = Vec::with_capacity(MAX_FRIEND_COUNT);
        read_friend_list(&mut friend_list, &friend_file)?;

        Ok(Self {
            ndm_wifi_event_handle,
            ndm_wifi_state: 0,
            wifi_connection_status: WiFiConnectionStatus::Idle,
            counter: 0,
            friend_list,
            account_config: get_my_account(&archive)?,
            my_data: get_my_data(&archive)?,
            my_online_activity: Default::default(),
            nat_properties: Default::default(),
            session_contexts: vec![],
            friend_key_list: [Default::default(); 100],
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
