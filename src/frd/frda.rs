use crate::FriendSysmodule;
use core::convert::From;
use ctr::{ctr_method, frd::GameKey, res::CtrResult, sysmodule::server::Service};
use no_std_io::{EndianRead, EndianWrite};
use num_enum::{FromPrimitive, IntoPrimitive};

#[derive(IntoPrimitive, FromPrimitive)]
#[repr(u16)]
pub enum FrdACommand {
    #[num_enum(default)]
    InvalidCommand = 0,
    // frd:u forward
    HasLoggedIn = 0x01,
    IsOnline = 0x02,
    Login = 0x03,
    Logout = 0x04,
    GetMyFriendKey = 0x05,
    GetMyPreference = 0x06,
    GetMyProfile = 0x07,
    GetMyPresence = 0x08,
    GetMyScreenName = 0x09,
    GetMyMii = 0x0A,
    GetMyLocalAccountId = 0x0B,
    GetMyPlayingGame = 0x0C,
    GetMyFavoriteGame = 0x0D,
    GetMyNcPrincipalId = 0x0E,
    GetMyComment = 0x0F,
    GetMyPassword = 0x10,
    GetFriendKeyList = 0x11,
    GetFriendPresence = 0x12,
    GetFriendScreenName = 0x13,
    GetFriendMii = 0x14,
    GetFriendProfile = 0x15,
    GetFriendRelationship = 0x16,
    GetFriendAttributeFlags = 0x17,
    GetFriendPlayingGame = 0x18,
    GetFriendFavoriteGame = 0x19,
    GetFriendInfo = 0x1A,
    IsIncludedInFriendList = 0x1B,
    UnscrambleLocalFriendCode = 0x1C,
    UpdateGameModeDescription = 0x1D,
    UpdateGameMode = 0x1E,
    SendInvitation = 0x1F,
    AttachToEventNotification = 0x20,
    SetNotificationMask = 0x21,
    GetEventNotification = 0x22,
    GetLastResponseResult = 0x23,
    PrincipalIdToFriendCode = 0x24,
    FriendCodeToPrincipalId = 0x25,
    IsValidFriendCode = 0x26,
    ResultToErrorCode = 0x27,
    RequestGameAuthentication = 0x28,
    GetGameAuthenticationData = 0x29,
    RequestServiceLocator = 0x2A,
    GetServiceLocatorData = 0x2B,
    DetectNatProperties = 0x2C,
    GetNatProperties = 0x2D,
    GetServerTimeInterval = 0x2E,
    AllowHalfAwake = 0x2F,
    GetServerTypes = 0x30,
    GetFriendComment = 0x31,
    SetClientSdkVersion = 0x32,
    GetMyApproachContext = 0x33,
    AddFriendWithApproach = 0x34,
    DecryptApproachContext = 0x35,
    GetExtendedNatProperties = 0x36,

    // frd:a exclusive
    CreateLocalAccount = 0x401,
    DeleteConfig = 0x402,
    SetLocalAccountId = 0x403,
    ResetAccountConfig = 0x404,
    HasUserData = 0x405,
    AddFriendOnline = 0x406,
    AddFriendOffline = 0x407,
    SetFriendDisplayName = 0x408,
    RemoveFriend = 0x409,
    SetPresenseGameKey = 0x40a,
    SetPrivacySettings = 0x40b,
    SetMyData = 0x40c,
    SetMyFavoriteGame = 0x40d,
    SetMyNCPrincipalId = 0x40e,
    SetPersonalComment = 0x40f,
    IncrementAccountConfigCounter = 0x410,
}

impl Service for FrdACommand {
    const ID: usize = 1;
    const NAME: &'static str = "frd:a";
    const MAX_SESSION_COUNT: i32 = 8;
}

#[derive(EndianRead, EndianWrite)]
struct CreateLocalAccountIn {
    local_account_id: u32,
    nasc_environment: u32,
    server_type_field_1: u32,
    server_type_field_2: u32,
}

#[ctr_method(cmd = "FrdACommand::CreateLocalAccount", normal = 0x1, translate = 0x0)]
fn create_local_account(
    _server: &mut FriendSysmodule,
    _session_index: usize,
    _input: CreateLocalAccountIn,
) -> CtrResult {
    // Stubbed so we don't write actual save data
    Ok(())
}

#[ctr_method(cmd = "FrdACommand::HasUserData", normal = 0x1, translate = 0x0)]
fn has_user_data(_server: &mut FriendSysmodule, _session_index: usize) -> CtrResult {
    Ok(())
}

#[ctr_method(cmd = "FrdACommand::SetPresenseGameKey", normal = 0x1, translate = 0x0)]
fn set_precense_game_key(
    server: &mut FriendSysmodule,
    _session_index: usize,
    playing_game: GameKey,
) -> CtrResult {
    server.context.my_online_activity.playing_game = playing_game;
    Ok(())
}

#[ctr_method(cmd = "FrdACommand::SetMyData", normal = 0x1, translate = 0x0)]
fn set_my_data(_server: &mut FriendSysmodule, _session_index: usize) -> CtrResult {
    // Stubbed so we don't write actual save data
    Ok(())
}
