#![no_std]
#![allow(incomplete_features)]
#![feature(alloc_error_handler)]
#![feature(start)]
#![feature(if_let_guard)]

extern crate alloc;

mod frd;
mod heap_allocator;
mod log;

use alloc::vec;
#[cfg(not(test))]
use core::{arch::asm, panic::PanicInfo};
use ctr::{
    ac, fs,
    http::httpc_init,
    ipc::WrittenCommand,
    match_ctr_route,
    memory::{MemoryBlock, MemoryPermission},
    ptm,
    res::CtrResult,
    srv, svc,
    sysmodule::{
        notification::NotificationManager,
        server::{Service, ServiceManager, ServiceRouter},
    },
};
use frd::{
    context::FriendServiceContext, frda::FrdACommand, frdn::FrdNCommand, frdu::FrdUCommand,
    notification::handle_sleep_notification,
};

/// Called after main exits to clean things up.
/// Used by 3ds toolchain.
#[no_mangle]
pub extern "C" fn __wrap_exit() {
    svc::exit_process();
}

#[repr(align(0x1000))]
struct HttpBuffer([u8; 0x1000]);

impl HttpBuffer {
    fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

static mut HTTP_BUFFER: HttpBuffer = HttpBuffer([0; 0x1000]);

/// Called before main to initialize the system.
/// Used by 3ds toolchain.
#[no_mangle]
pub extern "C" fn initSystem() {
    // This is safe because we're only supposed to use this one time
    // while initializing the system, which is happening right here.
    unsafe { heap_allocator::init_heap() };

    loop {
        match srv::init() {
            Ok(_) => break,
            Err(error_code) => {
                if error_code != 0xd88007fa {
                    panic!();
                }
            }
        };

        svc::sleep_thread(500000i64);
    }

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
}

#[cfg(not(test))]
#[panic_handler]
fn panic(panic: &PanicInfo<'_>) -> ! {
    if let Some(location) = panic.location() {
        let file = location.file();
        let slice = &file[file.len() - 7..];

        // Since we're about to break, storing a few u32s in these registers won't break us further.
        // In the future it might be helpful to disable this for release builds.
        unsafe {
            // r9 and r10 aren't used as frequently as the lower registers, so in most situations
            // we'll get more useful information by storing the last 4 characters of the file name
            // and the line number where we broke.
            let partial_file_name = *(slice.as_ptr() as *const u32);
            asm!("mov r9, {}", in(reg) partial_file_name);
            asm!("mov r10, {}", in(reg) location.line());
        }
    }

    svc::break_execution(svc::UserBreakType::Panic)
}

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn abort() -> ! {
    svc::break_execution(svc::UserBreakType::Panic)
}

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

#[start]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
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
            ptm::NotificationId::SleepRequested,
            handle_sleep_notification,
        )
        .unwrap();
    notification_manger
        .subscribe(ptm::NotificationId::GoingToSleep, handle_sleep_notification)
        .unwrap();
    notification_manger
        .subscribe(
            ptm::NotificationId::FullyWakingUp,
            handle_sleep_notification,
        )
        .unwrap();
    notification_manger
        .subscribe(
            ptm::NotificationId::Termination,
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

    svc::exit_process();
}
