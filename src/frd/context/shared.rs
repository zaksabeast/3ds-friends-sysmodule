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
use ctr::{
    frd::{FriendKey, GameKey, NatProperties, NotificationEvent},
    sysmodule::server::ServiceContext,
    Handle,
};

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

impl ServiceContext for FriendServiceContext {
    fn accept_session(&mut self) {
        let session_context = SessionContext::new();
        self.session_contexts.push(session_context);
    }

    fn close_session(&mut self, session_index: usize) {
        self.session_contexts.remove(session_index);
    }
}
