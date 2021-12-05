use super::FriendServiceContext;
use crate::frd::{
    save::{
        account::AccountConfig,
        friend_list::{FriendEntry, MAX_FRIEND_COUNT},
        my_data::MyData,
    },
    wifi::WiFiConnectionStatus,
};
use alloc::{vec, vec::Vec};
use core::convert::TryInto;
use ctr::{
    frd::FriendKey,
    fs::{ArchiveId, File, FsArchive, FsPath, OpenFlags},
    result::CtrResult,
    result::GenericResultCode,
    safe_transmute::transmute_one_pedantic,
    svc,
    svc::EventResetType,
};
use safe_transmute::{transmute_to_bytes, TriviallyTransmutable};

fn get_my_account(archive: &FsArchive) -> CtrResult<AccountConfig> {
    let account_path: FsPath = "/1/account".try_into()?;
    let account_file: [u8; 88] = archive
        .open_file(&account_path, OpenFlags::Read)?
        .read(0, 88)?
        .try_into()
        .map_err(|_| GenericResultCode::TryFromBytes)?;

    AccountConfig::try_from_le_bytes(account_file)
}

fn get_my_data(archive: &FsArchive) -> CtrResult<MyData> {
    let my_data_path: FsPath = "/1/mydata".try_into()?;
    let my_data_file: [u8; 288] = archive
        .open_file(&my_data_path, OpenFlags::Read)?
        .read(0, 288)?
        .try_into()
        .map_err(|_| GenericResultCode::TryFromBytes)?;

    MyData::try_from_le_bytes(my_data_file)
}

fn read_friend_entry(friend_file: &File, index: u64) -> CtrResult<Option<FriendEntry>> {
    let friend_bytes = friend_file.read((index * 0x100) + 16, 0x100)?;

    if friend_bytes.len() == 0x100 {
        let result = transmute_one_pedantic::<FriendEntry>(&friend_bytes)?;
        Ok(Some(result))
    } else {
        Ok(None)
    }
}

fn read_friend_list(friend_list: &mut Vec<FriendEntry>, friend_file: &File) -> CtrResult<()> {
    for index in 0..MAX_FRIEND_COUNT {
        if let Some(friend_entry) = read_friend_entry(friend_file, index as u64)? {
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

        let save_archive_path: Vec<u32> = vec![0, 0x10032];
        let archive = FsArchive::new(ArchiveId::SystemSaveData, &save_archive_path.into())?;

        let friend_list_path: FsPath = "/1/friendlist".try_into()?;
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

    pub fn copy_into_session_static_buffer<T: TriviallyTransmutable>(
        &mut self,
        session_index: usize,
        data: &[T],
    ) -> &[u8] {
        let static_buffer = &mut self.session_contexts[session_index].static_buffer;
        static_buffer.clear();
        static_buffer.extend_from_slice(transmute_to_bytes(data));
        static_buffer
    }
}
