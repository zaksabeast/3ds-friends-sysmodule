use super::{frda::FrdACommand, result::FrdErrorCode, utils};
use crate::{
    frd::{
        online_play::{
            authentication::{create_game_login_request, GameAuthenticationData},
            locate::{create_game_service_locate_request, ServiceLocateData},
        },
        save::friend_list::MAX_FRIEND_COUNT,
    },
    FriendSysmodule,
};
use alloc::{str, vec, vec::Vec};
use core::{cmp::min, convert::From};
use ctr::{
    ctr_method,
    frd::{
        ExpandedFriendPresence, FriendComment, FriendInfo, FriendKey, FriendPresence,
        FriendProfile, GameKey, Mii, ScrambledFriendCode, ScreenName, TrivialCharacterSet,
    },
    ipc::{BufferRights, Command, CurrentProcessId, Handles, PermissionBuffer, StaticBuffer},
    result::CtrResult,
    svc,
    sysmodule::server::Service,
    time::calculate_time_difference_from_now,
    utils::cstring::parse_null_terminated_str,
};
use no_std_io::{Cursor, EndianRead, EndianWrite, StreamContainer, StreamWriter};
use num_enum::{FromPrimitive, IntoPrimitive};

#[derive(IntoPrimitive, FromPrimitive)]
#[repr(u16)]
pub enum FrdUCommand {
    #[num_enum(default)]
    InvalidCommand = 0,
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
}

impl Service for FrdUCommand {
    const ID: usize = 0;
    const NAME: &'static str = "frd:u";
    const MAX_SESSION_COUNT: i32 = 8;
}

#[ctr_method(cmd = "FrdUCommand::HasLoggedIn", normal = 0x2, translate = 0x0)]
#[ctr_method(cmd = "FrdACommand::HasLoggedIn", normal = 0x2, translate = 0x0)]
fn has_logged_in(_server: &mut FriendSysmodule, _session_index: usize) -> CtrResult<u32> {
    Ok(true as u32)
}

#[ctr_method(cmd = "FrdUCommand::IsOnline", normal = 0x2, translate = 0x0)]
#[ctr_method(cmd = "FrdACommand::IsOnline", normal = 0x2, translate = 0x0)]
fn is_online(_server: &mut FriendSysmodule, _session_index: usize) -> CtrResult<u32> {
    Ok(true as u32)
}

#[ctr_method(cmd = "FrdUCommand::Login", normal = 0x1, translate = 0x0)]
#[ctr_method(cmd = "FrdACommand::Login", normal = 0x1, translate = 0x0)]
fn login(_server: &mut FriendSysmodule, _session_index: usize, event_handle: Handles) -> CtrResult {
    if let Some(handle) = event_handle.into_handle() {
        svc::signal_event(&handle)?;
    }
    Ok(())
}

#[ctr_method(cmd = "FrdUCommand::Logout", normal = 0x1, translate = 0x0)]
#[ctr_method(cmd = "FrdACommand::Logout", normal = 0x1, translate = 0x0)]
fn logout(_server: &mut FriendSysmodule, _session_index: usize) -> CtrResult {
    Ok(())
}

#[ctr_method(cmd = "FrdUCommand::GetMyFriendKey", normal = 0x5, translate = 0x0)]
#[ctr_method(cmd = "FrdACommand::GetMyFriendKey", normal = 0x5, translate = 0x0)]
fn get_my_friend_key(server: &mut FriendSysmodule, _session_index: usize) -> CtrResult<FriendKey> {
    Ok(FriendKey {
        local_friend_code: server.context.account_config.local_friend_code,
        padding: 0,
        principal_id: server.context.account_config.principal_id,
    })
}

#[derive(EndianRead, EndianWrite)]
struct GetMyPreferenceOut {
    is_public_mode: u32,
    is_show_game_mode: u32,
    is_show_played_game: u32,
}

#[ctr_method(cmd = "FrdUCommand::GetMyPreference", normal = 0x4, translate = 0x0)]
#[ctr_method(cmd = "FrdACommand::GetMyPreference", normal = 0x4, translate = 0x0)]
fn get_my_preference(
    server: &mut FriendSysmodule,
    _session_index: usize,
) -> CtrResult<GetMyPreferenceOut> {
    Ok(GetMyPreferenceOut {
        is_public_mode: server.context.my_data.is_public_mode as u32,
        is_show_game_mode: server.context.my_data.is_show_game_mode as u32,
        is_show_played_game: server.context.my_data.is_show_played_game as u32,
    })
}

#[ctr_method(cmd = "FrdUCommand::GetMyProfile", normal = 0x3, translate = 0x0)]
#[ctr_method(cmd = "FrdACommand::GetMyProfile", normal = 0x3, translate = 0x0)]
fn get_my_profile(server: &mut FriendSysmodule, _session_index: usize) -> CtrResult<FriendProfile> {
    Ok(server.context.my_data.profile)
}

#[ctr_method(cmd = "FrdUCommand::GetMyPresence", normal = 0x1, translate = 0x2)]
#[ctr_method(cmd = "FrdACommand::GetMyPresence", normal = 0x1, translate = 0x2)]
fn get_my_presence(server: &mut FriendSysmodule, session_index: usize) -> CtrResult<StaticBuffer> {
    let presense = ExpandedFriendPresence::default();
    let static_buffer = server
        .context
        .copy_into_session_static_buffer(session_index, &[presense]);
    Ok(StaticBuffer::new(static_buffer, 0))
}

#[ctr_method(cmd = "FrdUCommand::GetMyScreenName", normal = 0xc, translate = 0x0)]
#[ctr_method(cmd = "FrdACommand::GetMyScreenName", normal = 0xc, translate = 0x0)]
fn get_my_screen_name(
    server: &mut FriendSysmodule,
    _session_index: usize,
) -> CtrResult<ScreenName> {
    let mut screen_name: [u16; 11] = [0; 11];
    server
        .context
        .my_data
        .screen_name
        .encode_utf16()
        .take(10)
        .enumerate()
        .for_each(|(index, short)| {
            screen_name[index] = short;
        });

    Ok(ScreenName::new(screen_name))
}

#[ctr_method(cmd = "FrdUCommand::GetMyMii", normal = 0x19, translate = 0x0)]
#[ctr_method(cmd = "FrdACommand::GetMyMii", normal = 0x19, translate = 0x0)]
fn get_my_mii(server: &mut FriendSysmodule, _session_index: usize) -> CtrResult<Mii> {
    Ok(server.context.my_data.mii)
}

#[ctr_method(
    cmd = "FrdUCommand::GetMyLocalAccountId",
    normal = 0x2,
    translate = 0x0
)]
#[ctr_method(
    cmd = "FrdACommand::GetMyLocalAccountId",
    normal = 0x2,
    translate = 0x0
)]
fn get_my_local_account_id(server: &mut FriendSysmodule, _session_index: usize) -> CtrResult<u32> {
    Ok(server.context.account_config.local_account_id)
}

#[ctr_method(cmd = "FrdUCommand::GetMyPlayingGame", normal = 0x5, translate = 0x0)]
#[ctr_method(cmd = "FrdACommand::GetMyPlayingGame", normal = 0x5, translate = 0x0)]
fn get_my_playing_game(server: &mut FriendSysmodule, _session_index: usize) -> CtrResult<GameKey> {
    let playing_game = server.context.my_online_activity.playing_game;
    Ok(GameKey {
        title_id: playing_game.title_id,
        version: playing_game.version,
        unk: playing_game.unk,
    })
}

#[ctr_method(cmd = "FrdUCommand::GetMyFavoriteGame", normal = 0x5, translate = 0x0)]
#[ctr_method(cmd = "FrdACommand::GetMyFavoriteGame", normal = 0x5, translate = 0x0)]
fn get_my_favorite_game(server: &mut FriendSysmodule, _session_index: usize) -> CtrResult<GameKey> {
    Ok(GameKey {
        title_id: server.context.my_data.my_favorite_game.title_id,
        version: server.context.my_data.my_favorite_game.version,
        unk: 0,
    })
}

#[ctr_method(cmd = "FrdUCommand::GetMyNcPrincipalId", normal = 0x2, translate = 0x0)]
#[ctr_method(cmd = "FrdACommand::GetMyNcPrincipalId", normal = 0x2, translate = 0x0)]
fn get_my_nc_principal_id(server: &mut FriendSysmodule, _session_index: usize) -> CtrResult<u32> {
    Ok(server.context.my_data.my_nc_principal_id)
}

#[ctr_method(cmd = "FrdUCommand::GetMyComment", normal = 0x12, translate = 0x0)]
#[ctr_method(cmd = "FrdACommand::GetMyComment", normal = 0x12, translate = 0x0)]
fn get_my_comment(server: &mut FriendSysmodule, _session_index: usize) -> CtrResult<FriendComment> {
    let mut comment_shorts: [u16; 17] = [0; 17];
    server
        .context
        .my_data
        .personal_comment
        .encode_utf16()
        .take(16)
        .enumerate()
        .for_each(|(index, short)| {
            comment_shorts[index] = short;
        });

    Ok(FriendComment::new(comment_shorts))
}

#[ctr_method(cmd = "FrdUCommand::GetMyPassword", normal = 0x1, translate = 0x2)]
#[ctr_method(cmd = "FrdACommand::GetMyPassword", normal = 0x1, translate = 0x2)]
fn get_my_password(server: &mut FriendSysmodule, session_index: usize) -> CtrResult<StaticBuffer> {
    let c_password =
        cstr_core::CString::new(server.context.account_config.nex_password.as_bytes())?;
    let c_password_bytes = c_password.to_bytes_with_nul();

    let static_buffer = server
        .context
        .copy_into_session_static_buffer(session_index, c_password_bytes);

    Ok(StaticBuffer::new(static_buffer, 0))
}

#[derive(EndianRead, EndianWrite)]
struct GetFriendKeyListIn {
    offset: u32,
    max: u32,
}

#[derive(EndianRead, EndianWrite)]
struct GetFriendKeyListOut {
    len: u32,
    friend_keys: StaticBuffer,
}

#[ctr_method(cmd = "FrdUCommand::GetFriendKeyList", normal = 0x2, translate = 0x2)]
#[ctr_method(cmd = "FrdACommand::GetFriendKeyList", normal = 0x2, translate = 0x2)]
fn get_friend_key_list(
    server: &mut FriendSysmodule,
    session_index: usize,
    input: GetFriendKeyListIn,
) -> CtrResult<GetFriendKeyListOut> {
    let friend_list_offset = input.offset as usize;
    let requested_number_of_friends = input.max as usize;

    let friend_keys = server.context.get_friend_keys();

    let start = min(friend_list_offset, friend_keys.len());
    let end = min(start + requested_number_of_friends, friend_keys.len());

    let sliced_friend_keys = &friend_keys[start..end].to_vec();
    let static_buffer = server
        .context
        .copy_into_session_static_buffer(session_index, sliced_friend_keys);

    Ok(GetFriendKeyListOut {
        len: sliced_friend_keys.len() as u32,
        friend_keys: StaticBuffer::new(static_buffer, 0),
    })
}

#[derive(EndianRead, EndianWrite)]
struct GetFriendPresenceIn {
    max_out: u32,
    friend_keys: StaticBuffer,
}

#[ctr_method(cmd = "FrdUCommand::GetFriendPresence", normal = 0x1, translate = 0x2)]
#[ctr_method(cmd = "FrdACommand::GetFriendPresence", normal = 0x1, translate = 0x2)]
fn get_friend_presence(
    server: &mut FriendSysmodule,
    session_index: usize,
    input: GetFriendPresenceIn,
) -> CtrResult<StaticBuffer> {
    <Command>::validate_header(0x120042u32)?;
    <Command>::validate_buffer_id(2, 0)?;

    let max_out_count = min(input.max_out as usize, MAX_FRIEND_COUNT);
    let result: Vec<FriendPresence> = vec![Default::default(); max_out_count];
    let static_buffer = server
        .context
        .copy_into_session_static_buffer(session_index, &result);

    Ok(StaticBuffer::new(static_buffer, 0))
}

#[derive(EndianRead, EndianWrite)]
struct GetFriendScreenNameIn {
    max_screen_name_out: u32,
    max_string_language_out: u32,
    friend_key_count: u32,
    // TODO: One of these might have to do with character sets
    unk1: u32,
    unk2: u32,
    friend_keys: StaticBuffer,
}

#[derive(EndianRead, EndianWrite)]
struct GetFriendScreenNameOut {
    friend_names: StaticBuffer,
    character_sets: StaticBuffer,
}

#[ctr_method(
    cmd = "FrdUCommand::GetFriendScreenName",
    normal = 0x1,
    translate = 0x4
)]
#[ctr_method(
    cmd = "FrdACommand::GetFriendScreenName",
    normal = 0x1,
    translate = 0x4
)]
fn get_friend_screen_name(
    server: &mut FriendSysmodule,
    session_index: usize,
    input: GetFriendScreenNameIn,
) -> CtrResult<GetFriendScreenNameOut> {
    <Command>::validate_header(0x130142u32)?;
    <Command>::validate_buffer_id(6, 0)?;

    let max_screen_name_out = input.max_screen_name_out as usize;
    let max_string_language_out = input.max_string_language_out as usize;
    let friend_key_count = min(input.friend_key_count as usize, MAX_FRIEND_COUNT);

    let max_out_count = min(
        friend_key_count,
        min(max_screen_name_out, max_string_language_out),
    );
    let friend_keys = unsafe { input.friend_keys.iter::<FriendKey>() };

    let result_size = max_out_count * core::mem::size_of::<ScreenName>()
        + max_out_count * core::mem::size_of::<TrivialCharacterSet>();
    let mut result: StreamContainer<Vec<u8>> =
        StreamContainer::new(Vec::with_capacity(result_size));
    let mut character_sets: Vec<TrivialCharacterSet> = Vec::with_capacity(max_out_count);

    friend_keys.take(max_out_count).for_each(|friend_key| {
        let (screen_name, character_set) =
            match server.context.get_friend_by_friend_key(&friend_key) {
                Some(friend) => (friend.screen_name, friend.character_set),
                None => (Default::default(), Default::default()),
            };
        result.checked_write_stream_le(&screen_name);
        character_sets.push(character_set)
    });

    let screen_name_buffer_length = result.get_index();

    character_sets.iter().for_each(|character_set| {
        result.checked_write_stream_le(character_set);
    });

    let static_buffer = server
        .context
        .copy_into_session_static_buffer(session_index, &result.into_raw());

    Ok(GetFriendScreenNameOut {
        friend_names: StaticBuffer::new(&static_buffer[..screen_name_buffer_length], 0),
        character_sets: StaticBuffer::new(&static_buffer[screen_name_buffer_length..], 1),
    })
}

#[derive(EndianRead, EndianWrite)]
struct GetFriendMiiIn {
    max_out_count: u32,
    friend_keys: StaticBuffer,
    friend_miis: PermissionBuffer,
}

#[ctr_method(cmd = "FrdUCommand::GetFriendMii", normal = 0x1, translate = 0x2)]
#[ctr_method(cmd = "FrdACommand::GetFriendMii", normal = 0x1, translate = 0x2)]
fn get_friend_mii(
    server: &mut FriendSysmodule,
    _session_index: usize,
    mut input: GetFriendMiiIn,
) -> CtrResult<PermissionBuffer> {
    <Command>::validate_header(0x140044u32)?;
    <Command>::validate_buffer_id(2, 0)?;

    let friend_keys = unsafe { input.friend_keys.iter::<FriendKey>() };
    let friend_miis_pointer = input.friend_miis.ptr();
    let friend_miis_len = input.friend_miis.len();
    let mut friend_miis = unsafe { input.friend_miis.as_write_stream() };
    let max_out_count = min(input.max_out_count as usize, MAX_FRIEND_COUNT);

    friend_keys.take(max_out_count).for_each(|friend_key| {
        let mii = server
            .context
            .get_friend_by_friend_key(&friend_key)
            .map(|friend| friend.mii)
            .unwrap_or_default();
        friend_miis.checked_write_stream_le(&mii);
    });

    Ok(PermissionBuffer::new(
        friend_miis_pointer,
        friend_miis_len,
        BufferRights::Write,
    ))
}

#[derive(EndianRead, EndianWrite)]
struct GetFriendProfileIn {
    max_out: u32,
    friend_keys: StaticBuffer,
}

#[ctr_method(cmd = "FrdUCommand::GetFriendProfile", normal = 0x1, translate = 0x2)]
#[ctr_method(cmd = "FrdACommand::GetFriendProfile", normal = 0x1, translate = 0x2)]
fn get_friend_profile(
    server: &mut FriendSysmodule,
    session_index: usize,
    input: GetFriendProfileIn,
) -> CtrResult<StaticBuffer> {
    <Command>::validate_header(0x150042u32)?;
    <Command>::validate_buffer_id(2, 0)?;

    let max_out_count = min(input.max_out as usize, MAX_FRIEND_COUNT);
    let friend_keys = unsafe { input.friend_keys.iter::<FriendKey>() };

    let result: Vec<FriendProfile> = friend_keys
        .take(max_out_count)
        .map(
            |friend_key| match server.context.get_friend_by_friend_key(&friend_key) {
                Some(friend) => friend.friend_profile,
                None => Default::default(),
            },
        )
        .collect();

    let static_buffer = server
        .context
        .copy_into_session_static_buffer(session_index, &result);

    Ok(StaticBuffer::new(static_buffer, 0))
}

#[derive(EndianRead, EndianWrite)]
struct GetFriendRelationshipIn {
    max_out: u32,
    friend_keys: StaticBuffer,
}

#[ctr_method(
    cmd = "FrdUCommand::GetFriendRelationship",
    normal = 0x1,
    translate = 0x2
)]
#[ctr_method(
    cmd = "FrdACommand::GetFriendRelationship",
    normal = 0x1,
    translate = 0x2
)]
fn get_friend_relationship(
    server: &mut FriendSysmodule,
    session_index: usize,
    input: GetFriendRelationshipIn,
) -> CtrResult<StaticBuffer> {
    <Command>::validate_header(0x160042u32)?;
    <Command>::validate_buffer_id(2, 0)?;

    let max_out_count = min(input.max_out as usize, MAX_FRIEND_COUNT);
    let friend_keys = unsafe { input.friend_keys.iter::<FriendKey>() };

    let result: Vec<u8> = friend_keys
        .take(max_out_count)
        .map(
            |friend_key| match server.context.get_friend_by_friend_key(&friend_key) {
                Some(friend) => friend.friend_relationship,
                None => 0,
            },
        )
        .collect();

    let static_buffer = server
        .context
        .copy_into_session_static_buffer(session_index, &result);

    Ok(StaticBuffer::new(static_buffer, 0))
}

#[derive(EndianRead, EndianWrite)]
struct GetFriendAttributeFlagsIn {
    max_out: u32,
    friend_keys: StaticBuffer,
}

#[ctr_method(
    cmd = "FrdUCommand::GetFriendAttributeFlags",
    normal = 0x1,
    translate = 0x2
)]
#[ctr_method(
    cmd = "FrdACommand::GetFriendAttributeFlags",
    normal = 0x1,
    translate = 0x2
)]
fn get_friend_attribute_flags(
    server: &mut FriendSysmodule,
    session_index: usize,
    input: GetFriendAttributeFlagsIn,
) -> CtrResult<StaticBuffer> {
    <Command>::validate_header(0x170042u32)?;
    <Command>::validate_buffer_id(2, 0)?;

    let max_out_count = min(input.max_out as usize, MAX_FRIEND_COUNT);
    let friend_keys = unsafe { input.friend_keys.iter::<FriendKey>() };

    let result: Vec<u32> = friend_keys
        .take(max_out_count)
        .map(
            |friend_key| match server.context.get_friend_by_friend_key(&friend_key) {
                Some(friend) => friend.get_attribute(),
                None => 0,
            },
        )
        .collect();

    let static_buffer = server
        .context
        .copy_into_session_static_buffer(session_index, &result);

    Ok(StaticBuffer::new(static_buffer, 0))
}

#[derive(EndianRead, EndianWrite)]
struct GetFriendPlayingGameIn {
    max_out: u32,
    friend_keys: StaticBuffer,
    game_keys: PermissionBuffer,
}

#[ctr_method(
    cmd = "FrdUCommand::GetFriendPlayingGame",
    normal = 0x1,
    translate = 0x2
)]
#[ctr_method(
    cmd = "FrdACommand::GetFriendPlayingGame",
    normal = 0x1,
    translate = 0x2
)]
fn get_friend_playing_game(
    _server: &mut FriendSysmodule,
    _session_index: usize,
    mut input: GetFriendPlayingGameIn,
) -> CtrResult<PermissionBuffer> {
    <Command>::validate_header(0x180044u32)?;
    <Command>::validate_buffer_id(2, 0)?;

    let max_out_count = min(input.max_out as usize, MAX_FRIEND_COUNT);
    let game_keys_pointer = input.game_keys.ptr();
    let mut game_keys = unsafe { input.game_keys.as_write_stream() };

    for _ in 0..max_out_count {
        let game_key = GameKey::default();
        game_keys.checked_write_stream_le(&game_key);
    }

    Ok(PermissionBuffer::new(
        game_keys_pointer,
        max_out_count,
        BufferRights::Write,
    ))
}

#[derive(EndianRead, EndianWrite)]
struct GetFriendFavoriteGameIn {
    max_out: u32,
    friend_keys: StaticBuffer,
}

#[ctr_method(
    cmd = "FrdUCommand::GetFriendFavoriteGame",
    normal = 0x1,
    translate = 0x2
)]
#[ctr_method(
    cmd = "FrdACommand::GetFriendFavoriteGame",
    normal = 0x1,
    translate = 0x2
)]
fn get_friend_favorite_game(
    server: &mut FriendSysmodule,
    session_index: usize,
    input: GetFriendFavoriteGameIn,
) -> CtrResult<StaticBuffer> {
    <Command>::validate_header(0x190042u32)?;
    <Command>::validate_buffer_id(2, 0)?;

    let max_out_count = min(input.max_out as usize, MAX_FRIEND_COUNT);
    let friend_keys = unsafe { input.friend_keys.iter::<FriendKey>() };

    let result: Vec<GameKey> = friend_keys
        .take(max_out_count)
        .map(
            |friend_key| match server.context.get_friend_by_friend_key(&friend_key) {
                Some(friend) => friend.favorite_game,
                None => Default::default(),
            },
        )
        .collect();

    let static_buffer = server
        .context
        .copy_into_session_static_buffer(session_index, &result);

    Ok(StaticBuffer::new(static_buffer, 0))
}

#[derive(EndianRead, EndianWrite)]
struct GetFriendInfoIn {
    max_out: u32,
    unk1: u32,
    // TODO: use this to filter some wide characters
    character_set: u32,
    friend_keys: StaticBuffer,
    friend_info_out: PermissionBuffer,
}

#[ctr_method(cmd = "FrdUCommand::GetFriendInfo", normal = 0x1, translate = 0x2)]
#[ctr_method(cmd = "FrdACommand::GetFriendInfo", normal = 0x1, translate = 0x2)]
fn get_friend_info(
    server: &mut FriendSysmodule,
    _session_index: usize,
    mut input: GetFriendInfoIn,
) -> CtrResult<PermissionBuffer> {
    <Command>::validate_header(0x1a00c4u32)?;
    <Command>::validate_buffer_id(4, 0)?;

    let friend_keys = unsafe { input.friend_keys.iter::<FriendKey>() };
    let friend_info_out_pointer = input.friend_info_out.ptr();
    let friend_out_len = input.friend_info_out.len();
    let mut friend_info_out = unsafe { input.friend_info_out.as_write_stream() };
    let max_out_count = min(input.max_out as usize, MAX_FRIEND_COUNT);

    friend_keys.take(max_out_count).for_each(|friend_key| {
        let friend_info = server
            .context
            .get_friend_by_friend_key(&friend_key)
            .map(|friend| FriendInfo::from(*friend))
            .unwrap_or_default();
        friend_info_out.checked_write_stream_le(&friend_info);
    });

    Ok(PermissionBuffer::new(
        friend_info_out_pointer,
        friend_out_len,
        BufferRights::Write,
    ))
}

#[ctr_method(
    cmd = "FrdUCommand::IsIncludedInFriendList",
    normal = 0x2,
    translate = 0x0
)]
#[ctr_method(
    cmd = "FrdACommand::IsIncludedInFriendList",
    normal = 0x2,
    translate = 0x0
)]
fn is_included_in_friend_list(
    server: &mut FriendSysmodule,
    _session_index: usize,
    friend_code: u64,
) -> CtrResult<u32> {
    let has_friend = server
        .context
        .friend_list
        .iter()
        .any(|friend| friend.friend_key.local_friend_code == friend_code);

    Ok(has_friend as u32)
}

#[derive(EndianRead, EndianWrite)]
struct UnscrambleLocalFriendCodeIn {
    max_out: u32,
    scrambled_friend_codes: StaticBuffer,
}

#[ctr_method(
    cmd = "FrdUCommand::UnscrambleLocalFriendCode",
    normal = 0x1,
    translate = 0x2
)]
#[ctr_method(
    cmd = "FrdACommand::UnscrambleLocalFriendCode",
    normal = 0x1,
    translate = 0x2
)]
fn unscramble_local_friend_code(
    server: &mut FriendSysmodule,
    session_index: usize,
    input: UnscrambleLocalFriendCodeIn,
) -> CtrResult<StaticBuffer> {
    <Command>::validate_header(0x1c0042u32)?;
    <Command>::validate_buffer_id(2, 1)?;

    let max_out_count = min(input.max_out as usize, MAX_FRIEND_COUNT);
    let scrambled_friend_codes =
        unsafe { input.scrambled_friend_codes.iter::<ScrambledFriendCode>() };

    let result: Vec<u64> = scrambled_friend_codes
        .take(max_out_count)
        .map(|scrambed_friend_code| {
            let friend_code = scrambed_friend_code.get_unscrambled_friend_code();
            let is_in_friend_list = server
                .context
                .friend_list
                .iter()
                .any(|friend| friend.friend_key.local_friend_code == friend_code);

            if is_in_friend_list {
                friend_code
            } else {
                0
            }
        })
        .collect();

    let static_buffer = server
        .context
        .copy_into_session_static_buffer(session_index, &result);

    Ok(StaticBuffer::new(static_buffer, 0))
}

#[ctr_method(
    cmd = "FrdUCommand::UpdateGameModeDescription",
    normal = 0x1,
    translate = 0x0
)]
#[ctr_method(
    cmd = "FrdACommand::UpdateGameModeDescription",
    normal = 0x1,
    translate = 0x0
)]
fn update_game_mode_description(_server: &mut FriendSysmodule, _session_index: usize) -> CtrResult {
    Ok(())
}

#[ctr_method(cmd = "FrdUCommand::UpdateGameMode", normal = 0x1, translate = 0x0)]
#[ctr_method(cmd = "FrdACommand::UpdateGameMode", normal = 0x1, translate = 0x0)]
fn update_game_mode(_server: &mut FriendSysmodule, _session_index: usize) -> CtrResult<u32> {
    Ok(0xc4e1)
}

#[ctr_method(cmd = "FrdUCommand::SendInvitation", normal = 0x1, translate = 0x0)]
#[ctr_method(cmd = "FrdACommand::SendInvitation", normal = 0x1, translate = 0x0)]
fn send_invitation(_server: &mut FriendSysmodule, _session_index: usize) -> CtrResult {
    Ok(())
}

#[ctr_method(
    cmd = "FrdUCommand::AttachToEventNotification",
    normal = 0x1,
    translate = 0x0
)]
#[ctr_method(
    cmd = "FrdACommand::AttachToEventNotification",
    normal = 0x1,
    translate = 0x0
)]
fn attach_to_event_notification(
    server: &mut FriendSysmodule,
    session_index: usize,
    client_event: u32,
) -> CtrResult {
    server.context.session_contexts[session_index].client_event = Some(client_event.into());
    Ok(())
}

#[ctr_method(
    cmd = "FrdUCommand::SetNotificationMask",
    normal = 0x1,
    translate = 0x0
)]
#[ctr_method(
    cmd = "FrdACommand::SetNotificationMask",
    normal = 0x1,
    translate = 0x0
)]
fn set_notification_mask(
    server: &mut FriendSysmodule,
    session_index: usize,
    notifixation_mask: u32,
) -> CtrResult {
    server.context.session_contexts[session_index].notification_mask = notifixation_mask;
    Ok(())
}

#[derive(EndianRead, EndianWrite)]
struct GetEventNotificationIn {
    max_out: u32,
    notifications_out: PermissionBuffer,
}

#[derive(EndianRead, EndianWrite)]
struct GetEventNotificationOut {
    unk: u32,
    out_len: u32,
    notifications: PermissionBuffer,
}

#[ctr_method(
    cmd = "FrdUCommand::GetEventNotification",
    normal = 0x3,
    translate = 0x2
)]
#[ctr_method(
    cmd = "FrdACommand::GetEventNotification",
    normal = 0x3,
    translate = 0x2
)]
fn get_event_notification(
    server: &mut FriendSysmodule,
    session_index: usize,
    mut input: GetEventNotificationIn,
) -> CtrResult<GetEventNotificationOut> {
    <Command>::validate_header(0x220042u32)?;

    let max_notification_count = min(input.max_out as usize, MAX_FRIEND_COUNT);
    let notification_out_pointer = input.notifications_out.ptr();
    let mut notification_out = unsafe { input.notifications_out.as_write_stream() };

    let client_event_queue = &mut server.context.session_contexts[session_index].client_event_queue;

    for notification in client_event_queue.iter().take(max_notification_count) {
        notification_out.checked_write_stream_le(notification);
    }

    client_event_queue.clear();

    Ok(GetEventNotificationOut {
        unk: 0,
        out_len: max_notification_count as u32,
        notifications: PermissionBuffer::new(
            notification_out_pointer,
            max_notification_count,
            BufferRights::Write,
        ),
    })
}

#[ctr_method(
    cmd = "FrdUCommand::GetLastResponseResult",
    normal = 0x1,
    translate = 0x0
)]
#[ctr_method(
    cmd = "FrdACommand::GetLastResponseResult",
    normal = 0x1,
    translate = 0x0
)]
fn get_last_response_result(_server: &mut FriendSysmodule, _session_index: usize) -> CtrResult {
    Ok(())
}

#[ctr_method(
    cmd = "FrdUCommand::PrincipalIdToFriendCode",
    normal = 0x3,
    translate = 0x0
)]
#[ctr_method(
    cmd = "FrdACommand::PrincipalIdToFriendCode",
    normal = 0x3,
    translate = 0x0
)]
fn principal_id_to_friend_code(
    _server: &mut FriendSysmodule,
    _session_index: usize,
    principal_id: u32,
) -> CtrResult<u64> {
    // Using ? for the implicit error conversion
    let result = utils::convert_principal_id_to_friend_code(principal_id)?;
    Ok(result)
}

#[ctr_method(
    cmd = "FrdUCommand::FriendCodeToPrincipalId",
    normal = 0x2,
    translate = 0x0
)]
#[ctr_method(
    cmd = "FrdACommand::FriendCodeToPrincipalId",
    normal = 0x2,
    translate = 0x0
)]
fn friend_code_to_principal_id(
    _server: &mut FriendSysmodule,
    _session_index: usize,
    friend_code: u64,
) -> CtrResult<u32> {
    // Using ? for the implicit error conversion
    let result = utils::convert_friend_code_to_principal_id(friend_code)?;
    Ok(result)
}

#[ctr_method(cmd = "FrdUCommand::IsValidFriendCode", normal = 0x2, translate = 0x0)]
#[ctr_method(cmd = "FrdACommand::IsValidFriendCode", normal = 0x2, translate = 0x0)]
fn is_valid_friend_code(
    _server: &mut FriendSysmodule,
    _session_index: usize,
    friend_code: u64,
) -> CtrResult<u32> {
    Ok(utils::validate_friend_code(friend_code) as u32)
}

#[ctr_method(cmd = "FrdUCommand::ResultToErrorCode", normal = 0x2, translate = 0x0)]
#[ctr_method(cmd = "FrdACommand::ResultToErrorCode", normal = 0x2, translate = 0x0)]
fn result_to_error_code(
    _server: &mut FriendSysmodule,
    _session_index: usize,
    result_code: i32,
) -> CtrResult<u32> {
    Ok(if result_code > -1 {
        0
    } else if (result_code & 0x3ff) == 0x101 {
        // TODO:
        // Incomplete, should return
        // 0x59D8 + some value or 0x4E20 + some value
        0x59D8
    } else {
        // TODO:
        // Incomplete, should return
        // 0x2710 + some value
        0x2710
    })
}

#[derive(EndianRead, EndianWrite)]
struct RequestGameAuthenticationDataIn {
    requesting_game_id: u32,
    ingamesn_bytes: [u8; 24],
    sdk_version_low: u32,
    sdk_version_high: u32,
    requesting_process_id: CurrentProcessId,
    event_handle: Handles,
}

#[ctr_method(
    cmd = "FrdUCommand::RequestGameAuthentication",
    normal = 0x1,
    translate = 0x0
)]
#[ctr_method(
    cmd = "FrdACommand::RequestGameAuthentication",
    normal = 0x1,
    translate = 0x0
)]
fn request_game_authentication(
    server: &mut FriendSysmodule,
    session_index: usize,
    input: RequestGameAuthenticationDataIn,
) -> CtrResult {
    <Command>::validate_header(0x280244u32)?;

    let request = create_game_login_request(
        &server.context,
        input.requesting_process_id.raw(),
        input.requesting_game_id,
        input.sdk_version_low as u8,
        input.sdk_version_high as u8,
        parse_null_terminated_str(&input.ingamesn_bytes),
    )?;

    let mut buffer: [u8; 312] = [0; 312];
    request.download_data_into_buffer(&mut buffer)?;

    let response_status_code = request.get_response_status_code()?;
    let buffer_str = str::from_utf8(&buffer)?
        .trim_end_matches(char::from(0))
        .trim_end_matches("\r\n");

    let authentication_response =
        GameAuthenticationData::from_fetched_response(buffer_str, response_status_code)?;

    server.context.session_contexts[session_index].last_game_authentication_response =
        Some(authentication_response);

    if let Some(handle) = input.event_handle.into_handle() {
        svc::signal_event(&handle)?;
    }

    Ok(())
}

#[ctr_method(
    cmd = "FrdUCommand::GetGameAuthenticationData",
    normal = 0x1,
    translate = 0x2
)]
#[ctr_method(
    cmd = "FrdACommand::GetGameAuthenticationData",
    normal = 0x1,
    translate = 0x2
)]
fn get_game_authentication_data(
    server: &mut FriendSysmodule,
    session_index: usize,
) -> CtrResult<StaticBuffer> {
    let last_game_authentication_response =
        server.context.session_contexts[session_index].last_game_authentication_response;

    let game_auth_data = last_game_authentication_response.ok_or(FrdErrorCode::MissingData)?;

    let static_buffer = server
        .context
        .copy_into_session_static_buffer(session_index, &[game_auth_data]);

    Ok(StaticBuffer::new(static_buffer, 0))
}

#[derive(EndianRead, EndianWrite)]
struct RequestServiceLocatorIn {
    requesting_game_id: u32,
    key_hash_bytes: [u8; 12],
    svc_bytes: [u8; 8],
    sdk_version_low: u32,
    sdk_version_high: u32,
    requesting_process_id: CurrentProcessId,
    event_handle: Handles,
}

#[ctr_method(
    cmd = "FrdUCommand::RequestServiceLocator",
    normal = 0x1,
    translate = 0x0
)]
#[ctr_method(
    cmd = "FrdACommand::RequestServiceLocator",
    normal = 0x1,
    translate = 0x0
)]
fn request_service_locator(
    server: &mut FriendSysmodule,
    session_index: usize,
    input: RequestServiceLocatorIn,
) -> CtrResult {
    <Command>::validate_header(0x2a0204u32)?;

    let request = create_game_service_locate_request(
        &server.context,
        input.requesting_process_id.raw(),
        input.requesting_game_id,
        input.sdk_version_low as u8,
        input.sdk_version_high as u8,
        parse_null_terminated_str(&input.key_hash_bytes),
        parse_null_terminated_str(&input.svc_bytes),
    )?;

    let mut buffer: [u8; 312] = [0; 312];
    request.download_data_into_buffer(&mut buffer)?;

    let response_status_code = request.get_response_status_code()?;
    let buffer_str = str::from_utf8(&buffer)?
        .trim_end_matches(char::from(0))
        .trim_end_matches("\r\n");

    let service_locator_response =
        ServiceLocateData::from_fetched_response(buffer_str, response_status_code)?;

    server.context.session_contexts[session_index].last_service_locator_response =
        Some(service_locator_response);

    let service_locator_timestamp = service_locator_response.timestamp.get_unix_timestamp();

    server.context.session_contexts[session_index].server_time_interval =
        calculate_time_difference_from_now(service_locator_timestamp);

    if let Some(handle) = input.event_handle.into_handle() {
        svc::signal_event(&handle)?;
    }

    Ok(())
}

#[ctr_method(
    cmd = "FrdUCommand::GetServiceLocatorData",
    normal = 0x1,
    translate = 0x2
)]
#[ctr_method(
    cmd = "FrdACommand::GetServiceLocatorData",
    normal = 0x1,
    translate = 0x2
)]
fn get_service_locator_data(
    server: &mut FriendSysmodule,
    session_index: usize,
) -> CtrResult<StaticBuffer> {
    let service_locator_response =
        server.context.session_contexts[session_index].last_service_locator_response;

    let service_locate_data = service_locator_response.ok_or(FrdErrorCode::MissingData)?;

    let static_buffer = server
        .context
        .copy_into_session_static_buffer(session_index, &[service_locate_data]);

    Ok(StaticBuffer::new(static_buffer, 0))
}

#[ctr_method(
    cmd = "FrdUCommand::DetectNatProperties",
    normal = 0x1,
    translate = 0x0
)]
#[ctr_method(
    cmd = "FrdACommand::DetectNatProperties",
    normal = 0x1,
    translate = 0x0
)]
fn detect_nat_properties(
    _server: &mut FriendSysmodule,
    _session_index: usize,
    event_handles: Handles,
) -> CtrResult {
    // Normally this should only signal once nat properties are fetched,
    // but we're not building online functionality at the moment, so
    // we'll signal it immediately.
    for event_handle in event_handles.into_handles().iter() {
        svc::signal_event(event_handle).unwrap();
    }

    Ok(())
}

#[derive(EndianRead, EndianWrite)]
struct GetNatPropertiesOut {
    unk1: u32,
    unk2: u32,
}

#[ctr_method(cmd = "FrdUCommand::GetNatProperties", normal = 0x3, translate = 0x0)]
#[ctr_method(cmd = "FrdACommand::GetNatProperties", normal = 0x3, translate = 0x0)]
fn get_nat_properties(
    server: &mut FriendSysmodule,
    _session_index: usize,
) -> CtrResult<GetNatPropertiesOut> {
    let nat_properties = &server.context.nat_properties;
    Ok(GetNatPropertiesOut {
        unk1: nat_properties.get_unk1() as u32,
        unk2: nat_properties.get_unk2() as u32,
    })
}

#[ctr_method(
    cmd = "FrdUCommand::GetServerTimeInterval",
    normal = 0x3,
    translate = 0x0
)]
#[ctr_method(
    cmd = "FrdACommand::GetServerTimeInterval",
    normal = 0x3,
    translate = 0x0
)]
fn get_server_time_interval(server: &mut FriendSysmodule, session_index: usize) -> CtrResult<u64> {
    Ok(server.context.session_contexts[session_index].server_time_interval)
}

#[ctr_method(cmd = "FrdUCommand::AllowHalfAwake", normal = 0x1, translate = 0x0)]
#[ctr_method(cmd = "FrdACommand::AllowHalfAwake", normal = 0x1, translate = 0x0)]
fn allow_half_awake(_server: &mut FriendSysmodule, _session_index: usize) -> CtrResult {
    Ok(())
}

#[derive(EndianRead, EndianWrite)]
struct GetServerTypesOut {
    nasc_environment: u32,
    server_type_1: u32,
    server_type_2: u32,
}

#[ctr_method(cmd = "FrdUCommand::GetServerTypes", normal = 0x4, translate = 0x0)]
#[ctr_method(cmd = "FrdACommand::GetServerTypes", normal = 0x4, translate = 0x0)]
fn get_server_types(
    server: &mut FriendSysmodule,
    _session_index: usize,
) -> CtrResult<GetServerTypesOut> {
    Ok(GetServerTypesOut {
        nasc_environment: server.context.account_config.nasc_environment as u32,
        server_type_1: server.context.account_config.server_type_1 as u32,
        server_type_2: server.context.account_config.server_type_2 as u32,
    })
}

#[derive(EndianRead, EndianWrite)]
struct GetFriendCommentIn {
    max_count: u32,
    unk1: u32,
    friend_keys: StaticBuffer,
}

#[ctr_method(cmd = "FrdUCommand::GetFriendComment", normal = 0x1, translate = 0x2)]
#[ctr_method(cmd = "FrdACommand::GetFriendComment", normal = 0x1, translate = 0x2)]
fn get_friend_comment(
    server: &mut FriendSysmodule,
    session_index: usize,
    input: GetFriendCommentIn,
) -> CtrResult<StaticBuffer> {
    <Command>::validate_header(0x310082u32)?;
    <Command>::validate_buffer_id(3, 0)?;

    let friend_key_count = min(input.max_count as usize, MAX_FRIEND_COUNT);
    let friend_keys = unsafe { input.friend_keys.iter::<FriendKey>() };

    let result: Vec<FriendComment> = friend_keys
        .take(friend_key_count)
        .map(
            |friend_key| match server.context.get_friend_by_friend_key(&friend_key) {
                Some(friend) => friend.comment,
                None => Default::default(),
            },
        )
        .collect();

    let static_buffer = server
        .context
        .copy_into_session_static_buffer(session_index, &result);

    Ok(StaticBuffer::new(static_buffer, 0))
}

#[derive(EndianRead, EndianWrite)]
struct SetClientSdkVersionIn {
    sdk_verion: u32,
    process_id: CurrentProcessId,
}

#[ctr_method(
    cmd = "FrdUCommand::SetClientSdkVersion",
    normal = 0x1,
    translate = 0x0
)]
#[ctr_method(
    cmd = "FrdACommand::SetClientSdkVersion",
    normal = 0x1,
    translate = 0x0
)]
fn set_client_sdk_version(
    server: &mut FriendSysmodule,
    session_index: usize,
    input: SetClientSdkVersionIn,
) -> CtrResult {
    <Command>::validate_header(0x320042u32)?;

    let session_context = &mut server.context.session_contexts[session_index];
    session_context.client_sdk_version = input.sdk_verion;
    session_context.process_id = input.process_id.raw();
    Ok(())
}

#[ctr_method(
    cmd = "FrdUCommand::GetMyApproachContext",
    normal = 0x1,
    translate = 0x0
)]
#[ctr_method(
    cmd = "FrdACommand::GetMyApproachContext",
    normal = 0x1,
    translate = 0x0
)]
fn get_my_approach_context(_server: &mut FriendSysmodule, _session_index: usize) -> CtrResult {
    Ok(())
}

#[ctr_method(
    cmd = "FrdUCommand::AddFriendWithApproach",
    normal = 0x1,
    translate = 0x0
)]
#[ctr_method(
    cmd = "FrdACommand::AddFriendWithApproach",
    normal = 0x1,
    translate = 0x0
)]
fn add_friend_with_approach(_server: &mut FriendSysmodule, _session_index: usize) -> CtrResult {
    Ok(())
}

#[ctr_method(
    cmd = "FrdUCommand::DecryptApproachContext",
    normal = 0x1,
    translate = 0x0
)]
#[ctr_method(
    cmd = "FrdACommand::DecryptApproachContext",
    normal = 0x1,
    translate = 0x0
)]
fn decrypt_approach_context(_server: &mut FriendSysmodule, _session_index: usize) -> CtrResult {
    Ok(())
}

#[derive(EndianRead, EndianWrite)]
struct GetExtendedNatPropertiesOut {
    unk1: u32,
    unk2: u32,
    unk3: u32,
}

#[ctr_method(
    cmd = "FrdUCommand::GetExtendedNatProperties",
    normal = 0x4,
    translate = 0x0
)]
#[ctr_method(
    cmd = "FrdACommand::GetExtendedNatProperties",
    normal = 0x4,
    translate = 0x0
)]
fn get_extended_nat_properties(
    server: &mut FriendSysmodule,
    _session_index: usize,
) -> CtrResult<GetExtendedNatPropertiesOut> {
    let nat_properties = &server.context.nat_properties;
    Ok(GetExtendedNatPropertiesOut {
        unk1: nat_properties.get_unk1() as u32,
        unk2: nat_properties.get_unk2() as u32,
        unk3: nat_properties.get_unk3() as u32,
    })
}
