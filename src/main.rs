#![no_std]
#![allow(incomplete_features)]
#![feature(alloc_error_handler)]
#![feature(start)]
#![feature(if_let_guard)]

extern crate alloc;

mod frd;
mod log;

use alloc::vec;
use ctr::{
    ac, fs,
    http::httpc_init,
    ipc::WrittenCommand,
    match_ctr_route,
    memory::{MemoryBlock, MemoryPermission},
    ptm_sysm,
    res::CtrResult,
    svc,
    sysmodule::{
        notification::NotificationManager,
        server::{Service, ServiceManager, ServiceRouter},
    },
};
use frd::{
    context::FriendServiceContext, frda::FrdACommand, frdn::FrdNCommand, frdu::FrdUCommand,
    notification::handle_sleep_notification,
};

#[repr(align(0x1000))]
struct HttpBuffer([u8; 0x1000]);

impl HttpBuffer {
    fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

static mut HTTP_BUFFER: HttpBuffer = HttpBuffer([0; 0x1000]);

fn handle_termination_notification(_notification: u32) -> CtrResult {
    svc::exit_process();
}

struct FriendSysmodule {
    context: FriendServiceContext,
}

impl FriendSysmodule {
    fn new() -> Self {
        Self {
            context: FriendServiceContext::new().unwrap(),
        }
    }
}

impl ServiceRouter for FriendSysmodule {
    fn handle_request(
        &mut self,
        service_id: usize,
        session_index: usize,
    ) -> CtrResult<WrittenCommand> {
        match_ctr_route!(
            FriendSysmodule,
            service_id,
            session_index,
            FrdNCommand::GetWiFiEvent,
            FrdNCommand::ConnectToWiFi,
            FrdNCommand::DisconnectFromWiFi,
            FrdNCommand::GetWiFiState,
            FrdACommand::HasLoggedIn,
            FrdACommand::IsOnline,
            FrdACommand::Login,
            FrdACommand::Logout,
            FrdACommand::GetMyFriendKey,
            FrdACommand::GetMyPreference,
            FrdACommand::GetMyProfile,
            FrdACommand::GetMyPresence,
            FrdACommand::GetMyScreenName,
            FrdACommand::GetMyMii,
            FrdACommand::GetMyLocalAccountId,
            FrdACommand::GetMyPlayingGame,
            FrdACommand::GetMyFavoriteGame,
            FrdACommand::GetMyNcPrincipalId,
            FrdACommand::GetMyComment,
            FrdACommand::GetMyPassword,
            FrdACommand::GetFriendKeyList,
            FrdACommand::GetFriendPresence,
            FrdACommand::GetFriendScreenName,
            FrdACommand::GetFriendMii,
            FrdACommand::GetFriendProfile,
            FrdACommand::GetFriendRelationship,
            FrdACommand::GetFriendAttributeFlags,
            FrdACommand::GetFriendPlayingGame,
            FrdACommand::GetFriendFavoriteGame,
            FrdACommand::GetFriendInfo,
            FrdACommand::IsIncludedInFriendList,
            FrdACommand::UnscrambleLocalFriendCode,
            FrdACommand::UpdateGameModeDescription,
            FrdACommand::UpdateGameMode,
            FrdACommand::SendInvitation,
            FrdACommand::AttachToEventNotification,
            FrdACommand::SetNotificationMask,
            FrdACommand::GetEventNotification,
            FrdACommand::GetLastResponseResult,
            FrdACommand::PrincipalIdToFriendCode,
            FrdACommand::FriendCodeToPrincipalId,
            FrdACommand::IsValidFriendCode,
            FrdACommand::ResultToErrorCode,
            FrdACommand::RequestGameAuthentication,
            FrdACommand::GetGameAuthenticationData,
            FrdACommand::RequestServiceLocator,
            FrdACommand::GetServiceLocatorData,
            FrdACommand::DetectNatProperties,
            FrdACommand::GetNatProperties,
            FrdACommand::GetServerTimeInterval,
            FrdACommand::AllowHalfAwake,
            FrdACommand::GetServerTypes,
            FrdACommand::GetFriendComment,
            FrdACommand::SetClientSdkVersion,
            FrdACommand::GetMyApproachContext,
            FrdACommand::AddFriendWithApproach,
            FrdACommand::DecryptApproachContext,
            FrdACommand::GetExtendedNatProperties,
            FrdACommand::CreateLocalAccount,
            FrdACommand::HasUserData,
            FrdACommand::SetPresenseGameKey,
            FrdACommand::SetMyData,
            FrdUCommand::HasLoggedIn,
            FrdUCommand::IsOnline,
            FrdUCommand::Login,
            FrdUCommand::Logout,
            FrdUCommand::GetMyFriendKey,
            FrdUCommand::GetMyPreference,
            FrdUCommand::GetMyProfile,
            FrdUCommand::GetMyPresence,
            FrdUCommand::GetMyScreenName,
            FrdUCommand::GetMyMii,
            FrdUCommand::GetMyLocalAccountId,
            FrdUCommand::GetMyPlayingGame,
            FrdUCommand::GetMyFavoriteGame,
            FrdUCommand::GetMyNcPrincipalId,
            FrdUCommand::GetMyComment,
            FrdUCommand::GetMyPassword,
            FrdUCommand::GetFriendKeyList,
            FrdUCommand::GetFriendPresence,
            FrdUCommand::GetFriendScreenName,
            FrdUCommand::GetFriendMii,
            FrdUCommand::GetFriendProfile,
            FrdUCommand::GetFriendRelationship,
            FrdUCommand::GetFriendAttributeFlags,
            FrdUCommand::GetFriendPlayingGame,
            FrdUCommand::GetFriendFavoriteGame,
            FrdUCommand::GetFriendInfo,
            FrdUCommand::IsIncludedInFriendList,
            FrdUCommand::UnscrambleLocalFriendCode,
            FrdUCommand::UpdateGameModeDescription,
            FrdUCommand::UpdateGameMode,
            FrdUCommand::SendInvitation,
            FrdUCommand::AttachToEventNotification,
            FrdUCommand::SetNotificationMask,
            FrdUCommand::GetEventNotification,
            FrdUCommand::GetLastResponseResult,
            FrdUCommand::PrincipalIdToFriendCode,
            FrdUCommand::FriendCodeToPrincipalId,
            FrdUCommand::IsValidFriendCode,
            FrdUCommand::ResultToErrorCode,
            FrdUCommand::RequestGameAuthentication,
            FrdUCommand::GetGameAuthenticationData,
            FrdUCommand::RequestServiceLocator,
            FrdUCommand::GetServiceLocatorData,
            FrdUCommand::DetectNatProperties,
            FrdUCommand::GetNatProperties,
            FrdUCommand::GetServerTimeInterval,
            FrdUCommand::AllowHalfAwake,
            FrdUCommand::GetServerTypes,
            FrdUCommand::GetFriendComment,
            FrdUCommand::SetClientSdkVersion,
            FrdUCommand::GetMyApproachContext,
            FrdUCommand::AddFriendWithApproach,
            FrdUCommand::DecryptApproachContext,
            FrdUCommand::GetExtendedNatProperties,
        )
    }

    fn accept_session(&mut self, _session_index: usize) {
        self.context.accept_session()
    }

    fn close_session(&mut self, session_index: usize) {
        self.context.close_session(session_index);
    }
}

#[ctr::ctr_start(heap_byte_size = 0x10000)]
fn main() {
    fs::init().unwrap();
    ac::init().unwrap();

    // This is safe as long as we're single threaded
    let aligned_buffer = unsafe { HTTP_BUFFER.as_mut_slice() };
    let memory_block = MemoryBlock::new(
        aligned_buffer,
        MemoryPermission::None,
        MemoryPermission::ReadWrite,
    )
    .expect("");
    httpc_init(memory_block).expect("HTTPC did not init");

    log::debug("\n\nStarted!");

    let router = FriendSysmodule::new();

    let services = vec![
        FrdUCommand::register().unwrap(),
        FrdACommand::register().unwrap(),
        FrdNCommand::register().unwrap(),
    ];

    log::debug("Setting up notification manager");

    let mut notification_manger = NotificationManager::new().unwrap();

    notification_manger
        .subscribe(
            ptm_sysm::NotificationId::SleepRequested,
            handle_sleep_notification,
        )
        .unwrap();
    notification_manger
        .subscribe(
            ptm_sysm::NotificationId::GoingToSleep,
            handle_sleep_notification,
        )
        .unwrap();
    notification_manger
        .subscribe(
            ptm_sysm::NotificationId::FullyWakingUp,
            handle_sleep_notification,
        )
        .unwrap();
    notification_manger
        .subscribe(
            ptm_sysm::NotificationId::Termination,
            handle_termination_notification,
        )
        .unwrap();

    // TODO:
    // notification_manger.subscribe(0x301, do_something);
    // notification_manger.subscribe(0x302, do_something);

    log::debug("Setting up service manager");
    let mut manager = ServiceManager::new(services, notification_manger, router);
    log::debug("Set up service manager");
    manager.run().unwrap();
}
