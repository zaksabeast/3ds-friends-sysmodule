use super::{context::FriendServiceContext, result::FrdErrorCode, utils};
use crate::frd::{
    online_play::{
        authentication::{create_game_login_request, GameAuthenticationData},
        locate::{create_game_service_locate_request, ServiceLocateData},
    },
    save::friend_list::MAX_FRIEND_COUNT,
};
use alloc::vec::Vec;
use core::{cmp::min, convert::From};
use ctr::{
    frd::{
        ExpandedFriendPresence, FriendComment, FriendInfo, FriendKey, FriendPresence,
        FriendProfile, GameKey, Mii, NotificationEvent, ScrambledFriendCode, ScreenName,
        TrivialCharacterSet,
    },
    ipc::{ThreadCommandBuilder, ThreadCommandParser},
    result::{GenericResultCode, ResultCode},
    svc,
    sysmodule::server::RequestHandlerResult,
    time::calculate_time_difference_from_now,
    utils::{cstring::parse_null_terminated_str, parse::str_from_utf8},
};
use num_enum::{FromPrimitive, IntoPrimitive};
use safe_transmute::{transmute_one_to_bytes, transmute_to_bytes};

#[derive(IntoPrimitive, FromPrimitive)]
#[repr(u16)]
enum FrdUCommand {
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

pub fn handle_frdu_request(
    context: &mut FriendServiceContext,
    mut command_parser: ThreadCommandParser,
    session_index: usize,
) -> RequestHandlerResult {
    let command_id = command_parser.get_command_id();

    match command_id.into() {
        FrdUCommand::HasLoggedIn => {
            let mut command = ThreadCommandBuilder::new(FrdUCommand::HasLoggedIn);
            command.push(ResultCode::success());
            command.push(true);
            Ok(command.build())
        }
        FrdUCommand::IsOnline => {
            let mut command = ThreadCommandBuilder::new(FrdUCommand::IsOnline);
            command.push(ResultCode::success());
            command.push(true);
            Ok(command.build())
        }
        FrdUCommand::Login => {
            if let Ok(event_handle) = command_parser.pop_handle() {
                svc::signal_event(&event_handle)?;
            }

            let mut command = ThreadCommandBuilder::new(FrdUCommand::Login);
            command.push(ResultCode::success());
            Ok(command.build())
        }
        FrdUCommand::Logout => {
            let mut command = ThreadCommandBuilder::new(FrdUCommand::Logout);
            command.push(ResultCode::success());
            Ok(command.build())
        }
        FrdUCommand::GetMyFriendKey => {
            let friend_key = FriendKey {
                local_friend_code: context.account_config.local_friend_code,
                padding: 0,
                principal_id: context.account_config.principal_id,
            };

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetMyFriendKey);
            command.push(ResultCode::success());
            command.push_struct(&friend_key);
            Ok(command.build())
        }
        FrdUCommand::GetMyPreference => {
            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetMyPreference);
            command.push(ResultCode::success());
            command.push(context.my_data.is_public_mode);
            command.push(context.my_data.is_show_game_mode);
            command.push(context.my_data.is_show_played_game);
            Ok(command.build())
        }
        FrdUCommand::GetMyProfile => {
            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetMyProfile);
            command.push(ResultCode::success());
            command.push_struct(&context.my_data.profile);
            Ok(command.build())
        }
        FrdUCommand::GetMyPresence => {
            let presense = ExpandedFriendPresence::default();
            let static_buffer = context.copy_into_session_static_buffer(session_index, &[presense]);

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetMyPresence);
            command.push(ResultCode::success());
            command.push_static_buffer(static_buffer, 0);
            Ok(command.build())
        }
        FrdUCommand::GetMyScreenName => {
            let mut screen_name: [u16; 11] = [0; 11];
            context
                .my_data
                .screen_name
                .encode_utf16()
                .take(10)
                .enumerate()
                .for_each(|(index, short)| {
                    screen_name[index] = short;
                });

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetMyScreenName);
            command.push(ResultCode::success());
            command.push_struct(&screen_name);
            Ok(command.build())
        }
        FrdUCommand::GetMyMii => {
            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetMyMii);
            command.push(ResultCode::success());
            command.push_struct(&context.my_data.mii);
            Ok(command.build())
        }
        FrdUCommand::GetMyLocalAccountId => {
            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetMyLocalAccountId);
            command.push(ResultCode::success());
            command.push(context.account_config.local_account_id);
            Ok(command.build())
        }
        FrdUCommand::GetMyPlayingGame => {
            let playing_game = context.my_online_activity.playing_game;
            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetMyPlayingGame);
            command.push(ResultCode::success());
            command.push_u64(playing_game.title_id);
            command.push(playing_game.version);
            command.push(playing_game.unk);
            Ok(command.build())
        }
        FrdUCommand::GetMyFavoriteGame => {
            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetMyFavoriteGame);
            command.push(ResultCode::success());
            command.push_u64(context.my_data.my_favorite_game.title_id);
            command.push(context.my_data.my_favorite_game.version);
            command.push(0u32); // unknown
            Ok(command.build())
        }
        FrdUCommand::GetMyNcPrincipalId => {
            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetMyNcPrincipalId);
            command.push(ResultCode::success());
            command.push(context.my_data.my_nc_principal_id);
            Ok(command.build())
        }
        FrdUCommand::GetMyComment => {
            let mut comment_shorts: [u16; 17] = [0; 17];
            context
                .my_data
                .personal_comment
                .encode_utf16()
                .take(16)
                .enumerate()
                .for_each(|(index, short)| {
                    comment_shorts[index] = short;
                });

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetMyComment);
            command.push(ResultCode::success());
            command.push_struct(&comment_shorts);
            Ok(command.build())
        }
        FrdUCommand::GetMyPassword => {
            let c_password =
                cstr_core::CString::new(context.account_config.nex_password.as_bytes())
                    .map_err(|_| GenericResultCode::InvalidString)?;
            let c_password_bytes = c_password.to_bytes_with_nul();

            let static_buffer =
                context.copy_into_session_static_buffer(session_index, c_password_bytes);

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetMyPassword);
            command.push(ResultCode::success());
            command.push_static_buffer(static_buffer, 0);
            Ok(command.build())
        }
        FrdUCommand::GetFriendKeyList => {
            let friend_list_offset = command_parser.pop() as usize;
            let requested_number_of_friends = command_parser.pop() as usize;

            let friend_keys = context.get_friend_keys();

            let start = min(friend_list_offset, friend_keys.len());
            let end = min(start + requested_number_of_friends, friend_keys.len());

            let sliced_friend_keys = &friend_keys[start..end];

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendKeyList);
            command.push(ResultCode::success());
            command.push(sliced_friend_keys.len() as u32);
            command.push_static_buffer(sliced_friend_keys, 0);
            Ok(command.build())
        }
        FrdUCommand::GetFriendPresence => {
            command_parser.validate_header(FrdUCommand::GetFriendPresence, 1, 2)?;
            command_parser.validate_buffer_id(2, 0)?;

            let max_out_count = min(command_parser.pop() as usize, MAX_FRIEND_COUNT);

            // This is safe since the buffer comes from kernel translation
            let friend_keys = unsafe { command_parser.pop_static_buffer::<FriendKey>()? };
            let result: Vec<FriendPresence> = friend_keys
                .iter()
                .take(max_out_count)
                .map(|_| Default::default())
                .collect();

            let static_buffer = context.copy_into_session_static_buffer(session_index, &result);

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendPresence);
            command.push(ResultCode::success());
            command.push_static_buffer(static_buffer, 0);
            Ok(command.build())
        }
        FrdUCommand::GetFriendScreenName => {
            command_parser.validate_header(FrdUCommand::GetFriendScreenName, 5, 2)?;
            command_parser.validate_buffer_id(6, 0)?;

            let max_screen_name_out = command_parser.pop() as usize;
            let max_string_language_out = command_parser.pop() as usize;
            let friend_key_count = min(command_parser.pop() as usize, MAX_FRIEND_COUNT);

            let max_out_count = min(
                friend_key_count,
                min(max_screen_name_out, max_string_language_out),
            );

            // TODO:
            // It looks like at least one of these has to do with character set
            // to compare between the user and the friend
            let _unk1 = command_parser.pop();
            let _unk2 = command_parser.pop();

            // This is safe since the buffer comes from kernel translation
            let friend_keys = unsafe { command_parser.pop_static_buffer::<FriendKey>()? };

            let result_size = max_out_count * core::mem::size_of::<ScreenName>()
                + max_out_count * core::mem::size_of::<TrivialCharacterSet>();
            let mut result: Vec<u8> = Vec::with_capacity(result_size);
            let mut character_sets: Vec<TrivialCharacterSet> = Vec::with_capacity(max_out_count);

            friend_keys
                .iter()
                .take(max_out_count)
                .for_each(|friend_key| {
                    let (screen_name, character_set) =
                        match context.get_friend_by_friend_key(friend_key) {
                            Some(friend) => (friend.screen_name, friend.character_set),
                            None => (Default::default(), Default::default()),
                        };
                    result.extend_from_slice(transmute_one_to_bytes(&screen_name));
                    character_sets.push(character_set)
                });

            let screen_name_buffer_length = result.len();

            result.extend_from_slice(transmute_to_bytes(&character_sets));

            let static_buffer = context.copy_into_session_static_buffer(session_index, &result);

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendScreenName);
            command.push(ResultCode::success());
            command.push_static_buffer(&static_buffer[..screen_name_buffer_length], 0);
            command.push_static_buffer(&static_buffer[screen_name_buffer_length..], 1);
            Ok(command.build())
        }
        FrdUCommand::GetFriendMii => {
            command_parser.validate_header(FrdUCommand::GetFriendMii, 1, 4)?;
            command_parser.validate_buffer_id(2, 0)?;

            let max_out_count = min(command_parser.pop() as usize, MAX_FRIEND_COUNT);

            // This is safe since the buffer comes from kernel translation
            let friend_keys = unsafe { command_parser.pop_static_buffer::<FriendKey>()? };
            // This is safe since the buffer comes from kernel translation
            let friend_miis = unsafe { command_parser.pop_mut_buffer::<Mii>()? };
            let friend_miis_pointer = friend_miis.as_mut_ptr();

            let mut out_count = 0;

            friend_keys
                .iter()
                .zip(friend_miis.iter_mut())
                .take(max_out_count)
                .for_each(|(friend_key, out_friend_mii)| {
                    match context.get_friend_by_friend_key(friend_key) {
                        Some(friend) => {
                            *out_friend_mii = friend.mii;
                        }
                        None => {
                            *out_friend_mii = Default::default();
                        }
                    }

                    out_count += 1;
                });

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendMii);
            command.push(ResultCode::success());
            unsafe { command.push_raw_write_buffer(friend_miis_pointer, out_count) };
            Ok(command.build())
        }
        FrdUCommand::GetFriendProfile => {
            command_parser.validate_header(FrdUCommand::GetFriendProfile, 1, 2)?;
            command_parser.validate_buffer_id(2, 0)?;

            let max_out_count = min(command_parser.pop() as usize, MAX_FRIEND_COUNT);

            // This is safe since the buffer comes from kernel translation
            let friend_keys = unsafe { command_parser.pop_static_buffer::<FriendKey>()? };
            let result: Vec<FriendProfile> = friend_keys
                .iter()
                .take(max_out_count)
                .map(
                    |friend_key| match context.get_friend_by_friend_key(friend_key) {
                        Some(friend) => friend.friend_profile,
                        None => Default::default(),
                    },
                )
                .collect();

            let static_buffer = context.copy_into_session_static_buffer(session_index, &result);

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendProfile);
            command.push(ResultCode::success());
            command.push_static_buffer(static_buffer, 0);
            Ok(command.build())
        }
        FrdUCommand::GetFriendRelationship => {
            command_parser.validate_header(FrdUCommand::GetFriendRelationship, 1, 2)?;
            command_parser.validate_buffer_id(2, 0)?;

            let max_out_count = min(command_parser.pop() as usize, MAX_FRIEND_COUNT);

            // This is safe since the buffer comes from kernel translation
            let friend_keys = unsafe { command_parser.pop_static_buffer::<FriendKey>()? };
            let result: Vec<u8> = friend_keys
                .iter()
                .take(max_out_count)
                .map(
                    |friend_key| match context.get_friend_by_friend_key(friend_key) {
                        Some(friend) => friend.friend_relationship,
                        None => 0,
                    },
                )
                .collect();

            let static_buffer = context.copy_into_session_static_buffer(session_index, &result);

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendRelationship);
            command.push(ResultCode::success());
            command.push_static_buffer(static_buffer, 0);
            Ok(command.build())
        }
        FrdUCommand::GetFriendAttributeFlags => {
            command_parser.validate_header(FrdUCommand::GetFriendAttributeFlags, 1, 2)?;
            command_parser.validate_buffer_id(2, 0)?;

            let max_out_count = min(command_parser.pop() as usize, MAX_FRIEND_COUNT);

            let friend_keys = unsafe { command_parser.pop_static_buffer::<FriendKey>()? };
            let result: Vec<u32> = friend_keys
                .iter()
                .take(max_out_count)
                .map(
                    |friend_key| match context.get_friend_by_friend_key(friend_key) {
                        Some(friend) => friend.get_attribute(),
                        None => 0,
                    },
                )
                .collect();

            let static_buffer = context.copy_into_session_static_buffer(session_index, &result);

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendAttributeFlags);
            command.push(ResultCode::success());
            command.push_static_buffer(static_buffer, 0);
            Ok(command.build())
        }
        FrdUCommand::GetFriendPlayingGame => {
            command_parser.validate_header(FrdUCommand::GetFriendPlayingGame, 1, 4)?;
            command_parser.validate_buffer_id(2, 0)?;

            let max_out_count = min(command_parser.pop() as usize, MAX_FRIEND_COUNT);

            // This is safe since the buffer comes from kernel translation
            let friend_keys = unsafe { command_parser.pop_static_buffer::<FriendKey>()? };
            // This is safe since the buffer comes from kernel translation
            let game_keys = unsafe { command_parser.pop_mut_buffer::<GameKey>()? };
            let game_keys_pointer = game_keys.as_mut_ptr();

            let mut out_count = 0;

            friend_keys
                .iter()
                .zip(game_keys.iter_mut())
                .take(max_out_count)
                .for_each(|(_, out_game_key)| {
                    *out_game_key = Default::default();
                    out_count += 1;
                });

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendPlayingGame);
            command.push(ResultCode::success());
            // This is safe since we're forwarding kernel translated data back to the client
            unsafe { command.push_raw_write_buffer(game_keys_pointer, out_count) };
            Ok(command.build())
        }
        FrdUCommand::GetFriendFavoriteGame => {
            command_parser.validate_header(FrdUCommand::GetFriendFavoriteGame, 1, 2)?;
            command_parser.validate_buffer_id(2, 0)?;

            let max_out_count = min(command_parser.pop() as usize, MAX_FRIEND_COUNT);

            // This is safe since the buffer comes from kernel translation
            let friend_keys = unsafe { command_parser.pop_static_buffer::<FriendKey>()? };
            let result: Vec<GameKey> = friend_keys
                .iter()
                .take(max_out_count)
                .map(
                    |friend_key| match context.get_friend_by_friend_key(friend_key) {
                        Some(friend) => friend.favorite_game,
                        None => Default::default(),
                    },
                )
                .collect();

            let static_buffer = context.copy_into_session_static_buffer(session_index, &result);

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendFavoriteGame);
            command.push(ResultCode::success());
            command.push_static_buffer(static_buffer, 0);
            Ok(command.build())
        }
        FrdUCommand::GetFriendInfo => {
            command_parser.validate_header(FrdUCommand::GetFriendInfo, 3, 4)?;
            command_parser.validate_buffer_id(4, 0)?;

            let max_out_count = min(command_parser.pop() as usize, MAX_FRIEND_COUNT);
            let _unk1 = command_parser.pop();
            // TODO: use this to filter some wide characters
            let _character_set = command_parser.pop();

            // This is safe since the buffer comes from kernel translation
            let friend_keys = unsafe { command_parser.pop_static_buffer::<FriendKey>()? };

            // This is safe since the buffer comes from kernel translation
            let friend_info_out = unsafe { command_parser.pop_mut_buffer::<FriendInfo>()? };
            let friend_info_out_pointer = friend_info_out.as_mut_ptr();

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendInfo);
            let mut out_size = 0;

            friend_keys
                .iter()
                .zip(friend_info_out.iter_mut())
                .take(max_out_count)
                .for_each(|(friend_key, out_friend_info)| {
                    let friend_info = context
                        .friend_list
                        .iter()
                        .find_map(|friend| {
                            if friend.friend_key.principal_id == friend_key.principal_id {
                                Some(FriendInfo::from(*friend))
                            } else {
                                None
                            }
                        })
                        .unwrap_or_default();
                    *out_friend_info = friend_info;
                    out_size += 1;
                });

            command.push(ResultCode::success());
            // This is safe since we're forwarding kernel translated data back to the client
            unsafe { command.push_raw_write_buffer(friend_info_out_pointer, out_size) };
            Ok(command.build())
        }
        FrdUCommand::IsIncludedInFriendList => {
            let friend_code = command_parser.pop_u64();

            let has_friend = context
                .friend_list
                .iter()
                .any(|friend| friend.friend_key.local_friend_code == friend_code);

            let mut command = ThreadCommandBuilder::new(FrdUCommand::IsIncludedInFriendList);
            command.push(ResultCode::success());
            command.push(has_friend);
            Ok(command.build())
        }
        FrdUCommand::UnscrambleLocalFriendCode => {
            command_parser.validate_header(FrdUCommand::UnscrambleLocalFriendCode, 1, 2)?;
            command_parser.validate_buffer_id(2, 1)?;

            let max_out_count = min(command_parser.pop() as usize, MAX_FRIEND_COUNT);
            // This is safe since the buffer comes from kernel translation
            let scrambled_friend_codes =
                unsafe { command_parser.pop_static_buffer::<ScrambledFriendCode>()? };

            let result: Vec<u64> = scrambled_friend_codes
                .iter()
                .take(max_out_count)
                .map(|scrambed_friend_code| {
                    let friend_code = scrambed_friend_code.get_unscrambled_friend_code();
                    let is_in_friend_list = context
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

            let static_buffer = context.copy_into_session_static_buffer(session_index, &result);

            let mut command = ThreadCommandBuilder::new(FrdUCommand::UnscrambleLocalFriendCode);
            command.push(ResultCode::success());
            command.push_static_buffer(static_buffer, 0);
            Ok(command.build())
        }
        FrdUCommand::UpdateGameModeDescription => {
            let mut command = ThreadCommandBuilder::new(FrdUCommand::UpdateGameModeDescription);
            command.push(ResultCode::success());
            Ok(command.build())
        }
        FrdUCommand::UpdateGameMode => {
            let mut command = ThreadCommandBuilder::new(FrdUCommand::UpdateGameMode);
            command.push(0xc4e1u32);
            Ok(command.build())
        }
        FrdUCommand::SendInvitation => {
            let mut command = ThreadCommandBuilder::new(FrdUCommand::SendInvitation);
            command.push(ResultCode::success());
            Ok(command.build())
        }
        FrdUCommand::AttachToEventNotification => {
            command_parser.validate_header(FrdUCommand::AttachToEventNotification, 0, 2)?;
            let client_event = command_parser.pop_handle()?;

            context.session_contexts[session_index].client_event = Some(client_event);

            let mut command = ThreadCommandBuilder::new(FrdUCommand::AttachToEventNotification);
            command.push(ResultCode::success());
            Ok(command.build())
        }
        FrdUCommand::SetNotificationMask => {
            context.session_contexts[session_index].notification_mask = command_parser.pop();

            let mut command = ThreadCommandBuilder::new(FrdUCommand::SetNotificationMask);
            command.push(ResultCode::success());
            Ok(command.build())
        }
        FrdUCommand::GetEventNotification => {
            command_parser.validate_header(FrdUCommand::GetEventNotification, 1, 2)?;

            let max_notification_count = command_parser.pop() as usize;
            // This is safe since the buffer comes from kernel translation
            let notification_out = unsafe { command_parser.pop_mut_buffer::<NotificationEvent>()? };
            let notification_out_pointer = notification_out.as_mut_ptr();

            let client_event_queue =
                &mut context.session_contexts[session_index].client_event_queue;

            let mut out_count: usize = 0;

            notification_out
                .iter_mut()
                .zip(client_event_queue.iter())
                .take(max_notification_count)
                .for_each(|(out, notification)| {
                    *out = *notification;
                    out_count += 1;
                });

            client_event_queue.clear();

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetEventNotification);
            command.push(ResultCode::success());
            command.push(0u32); // unknown
            command.push(out_count as u32);
            // This is safe since we're forwarding kernel translated data back to the client
            unsafe { command.push_raw_write_buffer(notification_out_pointer, out_count) };
            Ok(command.build())
        }
        FrdUCommand::GetLastResponseResult => {
            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetLastResponseResult);
            command.push(ResultCode::success());
            Ok(command.build())
        }
        FrdUCommand::PrincipalIdToFriendCode => {
            let principal_id = command_parser.pop();
            let friend_code_result = utils::convert_principal_id_to_friend_code(principal_id);

            let mut command = ThreadCommandBuilder::new(FrdUCommand::PrincipalIdToFriendCode);

            match friend_code_result {
                Ok(friend_code) => {
                    command.push(ResultCode::success());
                    command.push_u64(friend_code);
                }
                Err(error_code) => {
                    command.push(error_code as u32);
                    command.push_u64(0);
                }
            }
            Ok(command.build())
        }
        FrdUCommand::FriendCodeToPrincipalId => {
            let friend_code = command_parser.pop_u64();
            let principal_id_result = utils::convert_friend_code_to_principal_id(friend_code);

            let mut command = ThreadCommandBuilder::new(FrdUCommand::FriendCodeToPrincipalId);

            match principal_id_result {
                Ok(principal_id) => {
                    command.push(ResultCode::success());
                    command.push(principal_id);
                }
                Err(error_code) => {
                    command.push(error_code as u32);
                    command.push(0u32);
                }
            }
            Ok(command.build())
        }
        FrdUCommand::IsValidFriendCode => {
            let friend_code = command_parser.pop_u64();
            let is_valid = utils::validate_friend_code(friend_code);

            let mut command = ThreadCommandBuilder::new(FrdUCommand::IsValidFriendCode);
            command.push(ResultCode::success());
            command.push(is_valid);
            Ok(command.build())
        }
        FrdUCommand::ResultToErrorCode => {
            let result_code = command_parser.pop_i32();

            let mut command = ThreadCommandBuilder::new(FrdUCommand::ResultToErrorCode);

            if (result_code >> 10) & 0xff != 0x31 {
                command.push(FrdErrorCode::InvalidErrorCode);
                command.push(0u32);
            } else if result_code > -1 {
                command.push(ResultCode::success());
                command.push(0u32);
            } else if (result_code & 0x3ff) == 0x101 {
                // TODO:
                // Incomplete, should return
                // 0x59D8 + some value or 0x4E20 + some value
                command.push(ResultCode::success());
                command.push(0x59D8u32);
            } else {
                // TODO:
                // Incomplete, should return
                // 0x2710 + some value
                command.push(ResultCode::success());
                command.push(0x2710u32);
            }

            Ok(command.build())
        }
        FrdUCommand::RequestGameAuthentication => {
            command_parser.validate_header(FrdUCommand::RequestGameAuthentication, 9, 4)?;

            let requesting_game_id = command_parser.pop();
            let ingamesn_bytes = command_parser.pop_struct::<[u8; 24]>()?;
            let ingamesn = parse_null_terminated_str(&ingamesn_bytes);
            let sdk_version_low = command_parser.pop();
            let sdk_version_high = command_parser.pop();
            let requesting_process_id = command_parser.pop_and_validate_process_id()?;
            let event_handle = command_parser.pop_handle()?;

            let request = create_game_login_request(
                context,
                requesting_process_id,
                requesting_game_id,
                sdk_version_low as u8,
                sdk_version_high as u8,
                ingamesn,
            )?;

            let mut buffer: [u8; 312] = [0; 312];
            request.download_data_into_buffer(&mut buffer)?;

            let response_status_code = request.get_response_status_code()?;
            let buffer_str = str_from_utf8(&buffer)?
                .trim_end_matches(char::from(0))
                .trim_end_matches("\r\n");

            let authentication_response =
                GameAuthenticationData::from_fetched_response(buffer_str, response_status_code)?;

            context.session_contexts[session_index].last_game_authentication_response =
                Some(authentication_response);

            svc::signal_event(&event_handle)?;

            let mut command = ThreadCommandBuilder::new(FrdUCommand::RequestGameAuthentication);
            command.push(ResultCode::success());
            Ok(command.build())
        }
        FrdUCommand::GetGameAuthenticationData => {
            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetGameAuthenticationData);

            let last_game_authentication_response =
                context.session_contexts[session_index].last_game_authentication_response;

            let game_auth_data = match last_game_authentication_response {
                Some(last_game_authentication_response) => {
                    command.push(ResultCode::success());
                    last_game_authentication_response
                }
                None => {
                    command.push(FrdErrorCode::MissingData);
                    Default::default()
                }
            };

            let static_buffer =
                context.copy_into_session_static_buffer(session_index, &[game_auth_data]);

            command.push_static_buffer(static_buffer, 0);
            Ok(command.build())
        }
        FrdUCommand::RequestServiceLocator => {
            command_parser.validate_header(FrdUCommand::RequestServiceLocator, 8, 4)?;

            let requesting_game_id = command_parser.pop();
            let key_hash_bytes = command_parser.pop_struct::<[u8; 12]>()?;
            let key_hash = parse_null_terminated_str(&key_hash_bytes);
            let svc_bytes = command_parser.pop_struct::<[u8; 8]>()?;
            let svc = parse_null_terminated_str(&svc_bytes);
            let sdk_version_low = command_parser.pop();
            let sdk_version_high = command_parser.pop();
            let requesting_process_id = command_parser.pop_and_validate_process_id()?;
            let event_handle = command_parser.pop_handle()?;

            let request = create_game_service_locate_request(
                context,
                requesting_process_id,
                requesting_game_id,
                sdk_version_low as u8,
                sdk_version_high as u8,
                key_hash,
                svc,
            )?;

            let mut buffer: [u8; 312] = [0; 312];
            request.download_data_into_buffer(&mut buffer)?;

            let response_status_code = request.get_response_status_code()?;
            let buffer_str = str_from_utf8(&buffer)?
                .trim_end_matches(char::from(0))
                .trim_end_matches("\r\n");

            let service_locator_response =
                ServiceLocateData::from_fetched_response(buffer_str, response_status_code)?;

            context.session_contexts[session_index].last_service_locator_response =
                Some(service_locator_response);

            let service_locator_timestamp = service_locator_response.timestamp.get_unix_timestamp();

            context.session_contexts[session_index].server_time_interval =
                calculate_time_difference_from_now(service_locator_timestamp);

            svc::signal_event(&event_handle)?;

            let mut command = ThreadCommandBuilder::new(FrdUCommand::RequestServiceLocator);
            command.push(ResultCode::success());
            Ok(command.build())
        }
        FrdUCommand::GetServiceLocatorData => {
            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetServiceLocatorData);
            let service_locator_response =
                context.session_contexts[session_index].last_service_locator_response;

            let service_locate_data = match service_locator_response {
                Some(service_locator_response) => {
                    command.push(ResultCode::success());
                    service_locator_response
                }
                None => {
                    command.push(FrdErrorCode::MissingData);
                    Default::default()
                }
            };

            let static_buffer =
                context.copy_into_session_static_buffer(session_index, &[service_locate_data]);

            command.push_static_buffer(static_buffer, 0);
            Ok(command.build())
        }
        FrdUCommand::DetectNatProperties => {
            let event = command_parser.pop_handle().unwrap();

            // Normally this should only signal once nat properties are fetched,
            // but we're not building online functionality at the moment, so
            // we'll signal it immediately.
            svc::signal_event(&event).unwrap();

            let mut command = ThreadCommandBuilder::new(FrdUCommand::DetectNatProperties);
            command.push(ResultCode::success());
            Ok(command.build())
        }
        FrdUCommand::GetNatProperties => {
            let nat_properties = &context.nat_properties;

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetNatProperties);
            command.push(ResultCode::success());
            command.push(nat_properties.get_unk1());
            command.push(nat_properties.get_unk2());
            Ok(command.build())
        }
        FrdUCommand::GetServerTimeInterval => {
            let server_time_interval = context.session_contexts[session_index].server_time_interval;

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetServerTimeInterval);
            command.push(ResultCode::success());
            command.push_u64(server_time_interval);
            Ok(command.build())
        }
        FrdUCommand::AllowHalfAwake => {
            let mut command = ThreadCommandBuilder::new(FrdUCommand::AllowHalfAwake);
            command.push(ResultCode::success());
            Ok(command.build())
        }
        FrdUCommand::GetServerTypes => {
            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetServerTypes);
            command.push(ResultCode::success());
            command.push(context.account_config.nasc_environment as u32);
            command.push(context.account_config.server_type_1);
            command.push(context.account_config.server_type_2);
            Ok(command.build())
        }
        FrdUCommand::GetFriendComment => {
            command_parser.validate_header(FrdUCommand::GetFriendComment, 2, 2)?;
            command_parser.validate_buffer_id(3, 0)?;

            let friend_key_count = min(command_parser.pop() as usize, MAX_FRIEND_COUNT);

            // The second argument is ignored in the official sysmodule
            command_parser.pop();

            // This is safe since the buffer comes from kernel translation
            let friend_keys = unsafe { command_parser.pop_static_buffer::<FriendKey>()? };
            let result: Vec<FriendComment> = friend_keys
                .iter()
                .take(friend_key_count)
                .map(
                    |friend_key| match context.get_friend_by_friend_key(friend_key) {
                        Some(friend) => friend.comment,
                        None => Default::default(),
                    },
                )
                .collect();

            let static_buffer = context.copy_into_session_static_buffer(session_index, &result);

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendComment);
            command.push(ResultCode::success());
            command.push_static_buffer(static_buffer, 0);
            Ok(command.build())
        }
        FrdUCommand::SetClientSdkVersion => {
            command_parser.validate_header(FrdUCommand::SetClientSdkVersion, 1, 2)?;

            let session_context = &mut context.session_contexts[session_index];
            session_context.client_sdk_version = command_parser.pop();
            session_context.process_id = command_parser.pop_and_validate_process_id()?;

            let mut command = ThreadCommandBuilder::new(FrdUCommand::SetClientSdkVersion);
            command.push(ResultCode::success());
            Ok(command.build())
        }
        FrdUCommand::GetMyApproachContext => {
            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetMyApproachContext);
            command.push(ResultCode::success());
            Ok(command.build())
        }
        FrdUCommand::AddFriendWithApproach => {
            let mut command = ThreadCommandBuilder::new(FrdUCommand::AddFriendWithApproach);
            command.push(ResultCode::success());
            Ok(command.build())
        }
        FrdUCommand::DecryptApproachContext => {
            let mut command = ThreadCommandBuilder::new(FrdUCommand::DecryptApproachContext);
            command.push(ResultCode::success());
            Ok(command.build())
        }
        FrdUCommand::GetExtendedNatProperties => {
            let nat_properties = &context.nat_properties;

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetExtendedNatProperties);
            command.push(ResultCode::success());
            command.push(nat_properties.get_unk1());
            command.push(nat_properties.get_unk2());
            command.push(nat_properties.get_unk3());
            Ok(command.build())
        }
        FrdUCommand::InvalidCommand => {
            let mut command = ThreadCommandBuilder::new(FrdUCommand::InvalidCommand);
            command.push(FrdErrorCode::InvalidCommand);
            Ok(command.build())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use alloc::string::ToString;
    use ctr::frd::{FriendProfile, GameKey, Mii};
    use mocktopus::mocking::{MockResult, Mockable};

    mod login {
        use super::*;

        #[test]
        fn should_signal_the_event_given_to_it() {
            let mock_handle: u32 = 0x1234;

            svc::signal_event.mock_safe(move |handle| {
                // This is safe since it's a test, the handle isn't real and it won't be used for anything.
                let raw_handle = unsafe { handle.get_raw() };
                assert_eq!(raw_handle, mock_handle);
                MockResult::Return(Ok(()))
            });

            let mut context = FriendServiceContext::new().unwrap();
            let mut command = ThreadCommandBuilder::new(FrdUCommand::Login);
            command.push_raw_handle(mock_handle);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(result.validate_header(FrdUCommand::Login, 1, 0), Ok(()));
            assert_eq!(result.pop_result(), Ok(()));
        }

        #[test]
        fn should_not_signal_an_invalid_event() {
            svc::signal_event.mock_safe(move |_| panic!("Event should not have been signalled!"));

            let mut context = FriendServiceContext::new().unwrap();
            let mut command = ThreadCommandBuilder::new(FrdUCommand::Login);
            // We're pushing invalid parameters
            // rather than pushing a handle
            command.push(0x1234u32);
            command.push(0x1234u32);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(result.validate_header(FrdUCommand::Login, 1, 0), Ok(()));
            assert_eq!(result.pop_result(), Ok(()));
        }
    }

    mod get_my_friend_key {
        use super::*;

        #[test]
        fn get_my_friend_key() {
            let friend_key = FriendKey {
                principal_id: 0xAA,
                local_friend_code: 0xAAAAAAAABBBBBBBB,
                ..Default::default()
            };

            let mut context = FriendServiceContext::new().unwrap();
            context.account_config.principal_id = friend_key.principal_id;
            context.account_config.local_friend_code = friend_key.local_friend_code;

            let command = ThreadCommandBuilder::new(FrdUCommand::GetMyFriendKey);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetMyFriendKey, 5, 0),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(result.pop_struct::<FriendKey>().unwrap(), friend_key);
        }
    }

    mod get_my_preference {
        use super::*;

        #[test]
        fn should_return_the_is_public_mode_preference() {
            let mut context = FriendServiceContext::new().unwrap();
            context.my_data.is_public_mode = true;
            context.my_data.is_show_game_mode = false;
            context.my_data.is_show_played_game = false;

            let command = ThreadCommandBuilder::new(FrdUCommand::GetMyPreference);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetMyPreference, 4, 0),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(result.pop(), 1);
            assert_eq!(result.pop(), 0);
            assert_eq!(result.pop(), 0);
        }

        #[test]
        fn should_return_the_is_show_game_mode_preference() {
            let mut context = FriendServiceContext::new().unwrap();
            context.my_data.is_public_mode = false;
            context.my_data.is_show_game_mode = true;
            context.my_data.is_show_played_game = false;

            let command = ThreadCommandBuilder::new(FrdUCommand::GetMyPreference);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetMyPreference, 4, 0),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(result.pop(), 0);
            assert_eq!(result.pop(), 1);
            assert_eq!(result.pop(), 0);
        }

        #[test]
        fn should_return_the_is_show_played_game_preference() {
            let mut context = FriendServiceContext::new().unwrap();
            context.my_data.is_public_mode = false;
            context.my_data.is_show_game_mode = false;
            context.my_data.is_show_played_game = true;

            let command = ThreadCommandBuilder::new(FrdUCommand::GetMyPreference);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetMyPreference, 4, 0),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(result.pop(), 0);
            assert_eq!(result.pop(), 0);
            assert_eq!(result.pop(), 1);
        }
    }

    mod get_my_profile {
        use super::*;

        #[test]
        fn get_my_profile() {
            let profile = FriendProfile {
                region: 1,
                country: 2,
                area: 3,
                language: 4,
                platform: 5, // in reality, this will always be 2 since that means 3ds
                padding: [0; 3],
            };

            let mut context = FriendServiceContext::new().unwrap();
            context.my_data.profile = profile;

            let command = ThreadCommandBuilder::new(FrdUCommand::GetMyProfile);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetMyProfile, 3, 0),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(result.pop_struct::<FriendProfile>().unwrap(), profile);
        }
    }

    mod get_my_presense {
        use ctr::sysmodule::server::ServiceContext;

        use super::*;
        use alloc::vec;

        #[test]
        fn should_return_the_default_presense() {
            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();

            let command = ThreadCommandBuilder::new(FrdUCommand::GetMyPresence);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetMyPresence, 1, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));

            let presense: Vec<ExpandedFriendPresence> =
                unsafe { result.pop_static_buffer().unwrap().to_vec() };
            assert_eq!(presense, vec![Default::default()]);
        }
    }

    mod get_my_screen_name {
        use super::*;

        #[test]
        fn should_return_my_screen_name() {
            let mut context = FriendServiceContext::new().unwrap();
            context.my_data.screen_name = "Test User".to_string();

            let command = ThreadCommandBuilder::new(FrdUCommand::GetMyScreenName);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetMyScreenName, 7, 0),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(
                result.pop_struct::<ScreenName>().unwrap(),
                "Test User".into()
            );
        }

        #[test]
        fn should_be_able_to_handle_a_screen_name_of_11_chars() {
            let mut context = FriendServiceContext::new().unwrap();
            context.my_data.screen_name = "s.len == 11".to_string();

            let command = ThreadCommandBuilder::new(FrdUCommand::GetMyScreenName);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetMyScreenName, 7, 0),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(
                result.pop_struct::<ScreenName>().unwrap(),
                "s.len == 11".into()
            );
        }

        #[test]
        fn should_truncate_screen_names_to_11_chars() {
            let mut context = FriendServiceContext::new().unwrap();
            context.my_data.screen_name = "This is more than 11 chars".to_string();

            let command = ThreadCommandBuilder::new(FrdUCommand::GetMyScreenName);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetMyScreenName, 7, 0),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(
                result.pop_struct::<ScreenName>().unwrap(),
                "This is mor".into()
            );
        }
    }

    mod get_my_mii {
        use super::*;

        #[test]
        fn get_my_mii() {
            let mii = Mii::new([
                0x22, 0x22, 0x11, 0x11, 0x44, 0x44, 0x33, 0x33, 0x66, 0x66, 0x55, 0x55, 0x88, 0x88,
                0x77, 0x77, 0xAA, 0xAA, 0x99, 0x99, 0xCC, 0xCC, 0xBB, 0xBB, 0xEE, 0xEE, 0xDD, 0xDD,
                0x00, 0x00, 0xFF, 0xFF, 0x22, 0x22, 0x11, 0x11, 0x44, 0x44, 0x33, 0x33, 0x66, 0x66,
                0x55, 0x55, 0x88, 0x88, 0x77, 0x77, 0xAA, 0xAA, 0x99, 0x99, 0xCC, 0xCC, 0xBB, 0xBB,
                0xEE, 0xEE, 0xDD, 0xDD, 0x00, 0x00, 0xFF, 0xFF, 0x22, 0x22, 0x11, 0x11, 0x44, 0x44,
                0x33, 0x33, 0x66, 0x66, 0x55, 0x55, 0x88, 0x88, 0x77, 0x77, 0xAA, 0xAA, 0x99, 0x99,
                0xCC, 0xCC, 0xBB, 0xBB, 0xEE, 0xEE, 0xDD, 0xDD, 0x00, 0x00, 0xFF, 0xFF,
            ]);

            let mut context = FriendServiceContext::new().unwrap();
            context.my_data.mii = mii;

            let command = ThreadCommandBuilder::new(FrdUCommand::GetMyMii);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(result.validate_header(FrdUCommand::GetMyMii, 25, 0), Ok(()));
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(result.pop_struct::<Mii>().unwrap(), mii);
        }
    }

    mod get_my_local_account_id {
        use super::*;

        #[test]
        fn get_my_local_account_id() {
            let mut context = FriendServiceContext::new().unwrap();
            context.account_config.local_account_id = 1;

            let command = ThreadCommandBuilder::new(FrdUCommand::GetMyLocalAccountId);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetMyLocalAccountId, 2, 0),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(result.pop(), 1);
        }
    }

    mod get_my_favorite_game {
        use super::*;

        #[test]
        fn get_my_favorite_game() {
            let game_key = GameKey {
                title_id: 0x1122334455667788,
                version: 0xaabbccdd,
                unk: 0,
            };

            let mut context = FriendServiceContext::new().unwrap();
            context.my_data.my_favorite_game = game_key;

            let command = ThreadCommandBuilder::new(FrdUCommand::GetMyFavoriteGame);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetMyFavoriteGame, 5, 0),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(result.pop_struct::<GameKey>().unwrap(), game_key);
        }
    }

    mod get_my_nc_principal_id {
        use super::*;

        #[test]
        fn get_my_nc_principal_id() {
            let mut context = FriendServiceContext::new().unwrap();
            context.my_data.my_nc_principal_id = 0xaabbccdd;

            let command = ThreadCommandBuilder::new(FrdUCommand::GetMyNcPrincipalId);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetMyNcPrincipalId, 2, 0),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(result.pop(), 0xaabbccdd);
        }
    }

    mod get_my_comment {
        use super::*;

        #[test]
        fn should_return_my_comment() {
            let mut context = FriendServiceContext::new().unwrap();
            context.my_data.personal_comment = "Hello!".to_string();

            let command = ThreadCommandBuilder::new(FrdUCommand::GetMyComment);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetMyComment, 10, 0),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(
                result.pop_struct::<FriendComment>().unwrap(),
                "Hello!".into()
            );
        }

        #[test]
        fn should_be_able_to_handle_a_comment_of_16_chars() {
            let mut context = FriendServiceContext::new().unwrap();
            context.my_data.personal_comment = "This is 16 chars".to_string();

            let command = ThreadCommandBuilder::new(FrdUCommand::GetMyComment);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetMyComment, 10, 0),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(
                result.pop_struct::<FriendComment>().unwrap(),
                "This is 16 chars".into()
            );
        }

        #[test]
        fn should_truncate_comments_to_16_chars() {
            let mut context = FriendServiceContext::new().unwrap();
            context.my_data.personal_comment = "This is more than 16 chars".to_string();

            let command = ThreadCommandBuilder::new(FrdUCommand::GetMyComment);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetMyComment, 10, 0),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(
                result.pop_struct::<FriendComment>().unwrap(),
                "This is more tha".into()
            );
        }
    }

    mod get_my_password {
        use super::*;
        use alloc::vec;
        use ctr::sysmodule::server::ServiceContext;

        #[test]
        fn should_return_the_null_terminated_user_password() {
            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.account_config.nex_password = "test".to_string();

            let command = ThreadCommandBuilder::new(FrdUCommand::GetMyPassword);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetMyPassword, 1, 2),
                Ok(())
            );
            assert_eq!(result.validate_buffer_id(2, 0), Ok(()));
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(
                context.get_session_static_buffer(0),
                vec![0x74, 0x65, 0x73, 0x74, 0x00]
            );
        }
    }

    mod get_friend_list {
        use super::*;
        use crate::frd::save::friend_list::FriendEntry;
        use alloc::vec;
        use ctr::sysmodule::server::ServiceContext;

        #[test]
        fn should_return_friend_keys_for_each_friend() {
            let friend_1: FriendEntry = Default::default();
            let friend_2: FriendEntry = Default::default();

            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.friend_list.clear();
            context.friend_list.push(friend_1);
            context.friend_list.push(friend_2);

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendKeyList);
            command.push(0u32);
            command.push(100u32);
            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetFriendKeyList, 2, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(result.pop(), 2);

            let result_friend_keys: Vec<FriendKey> =
                unsafe { result.pop_static_buffer().unwrap().to_vec() };
            assert_eq!(
                result_friend_keys,
                vec![friend_1.friend_key, friend_2.friend_key]
            );
        }

        #[test]
        fn should_limit_the_friend_keys_to_the_provided_max_number() {
            let friend_1: FriendEntry = Default::default();
            let friend_2: FriendEntry = Default::default();

            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.friend_list.clear();
            context.friend_list.push(friend_1);
            context.friend_list.push(friend_2);

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendKeyList);
            command.push(0u32);
            command.push(1u32);
            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetFriendKeyList, 2, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(result.pop(), 1);

            let result_friend_keys: Vec<FriendKey> =
                unsafe { result.pop_static_buffer().unwrap().to_vec() };
            assert_eq!(result_friend_keys, vec![friend_1.friend_key]);
        }

        #[test]
        fn should_start_at_the_provided_offset() {
            let friend_1: FriendEntry = Default::default();
            let friend_2: FriendEntry = Default::default();

            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.friend_list.clear();
            context.friend_list.push(friend_1);
            context.friend_list.push(friend_2);

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendKeyList);
            command.push(1u32);
            command.push(100u32);
            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetFriendKeyList, 2, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(result.pop(), 1);

            let result_friend_keys: Vec<FriendKey> =
                unsafe { result.pop_static_buffer().unwrap().to_vec() };
            assert_eq!(result_friend_keys, vec![friend_2.friend_key]);
        }

        #[test]
        fn should_return_nothing_if_the_offset_is_greater_than_the_friend_list_length() {
            let friend_1: FriendEntry = Default::default();
            let friend_2: FriendEntry = Default::default();

            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.friend_list.clear();
            context.friend_list.push(friend_1);
            context.friend_list.push(friend_2);

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendKeyList);
            command.push(2u32);
            command.push(100u32);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetFriendKeyList, 2, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(result.pop(), 0);

            let comments: Vec<FriendKey> = unsafe { result.pop_static_buffer().unwrap().to_vec() };
            assert_eq!(comments.len(), 0);
        }

        #[test]
        fn should_return_nothing_if_there_are_no_friends() {
            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.friend_list.clear();

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendKeyList);
            command.push(0u32);
            command.push(100u32);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetFriendKeyList, 2, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(result.pop(), 0);

            let comments: Vec<FriendKey> = unsafe { result.pop_static_buffer().unwrap().to_vec() };
            assert_eq!(comments.len(), 0);
        }
    }

    mod get_friend_presence {
        use super::*;
        use alloc::vec;
        use ctr::sysmodule::server::ServiceContext;

        #[test]
        fn should_return_the_default_presense_for_each_friend_key() {
            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();

            let friend_key_list: [FriendKey; 2] = [Default::default(); 2];
            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendPresence);
            command.push(friend_key_list.len() as u32);
            command.push_static_buffer(&friend_key_list, 0);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetFriendPresence, 1, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));

            let presense: Vec<FriendPresence> =
                unsafe { result.pop_static_buffer().unwrap().to_vec() };
            assert_eq!(presense.len(), 2);
            assert_eq!(presense[0], Default::default());
            assert_eq!(presense[1], Default::default());
        }

        #[test]
        fn should_limit_the_out_count_to_the_number_of_friend_keys() {
            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();

            let friend_key_list: [FriendKey; 2] = [Default::default(); 2];
            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendPresence);
            command.push(20u32);
            command.push_static_buffer(&friend_key_list, 0);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetFriendPresence, 1, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));

            let presense: Vec<FriendPresence> =
                unsafe { result.pop_static_buffer().unwrap().to_vec() };
            assert_eq!(presense.len(), 2);
            assert_eq!(presense[0], Default::default());
            assert_eq!(presense[1], Default::default());
        }

        #[test]
        fn should_limit_the_out_count_to_the_given_out_count() {
            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();

            let friend_key_list: [FriendKey; 2] = [Default::default(); 2];
            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendPresence);
            command.push(1u32);
            command.push_static_buffer(&friend_key_list, 0);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetFriendPresence, 1, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));

            let presense: Vec<FriendPresence> =
                unsafe { result.pop_static_buffer().unwrap().to_vec() };
            assert_eq!(presense.len(), 1);
            assert_eq!(presense[0], Default::default());
        }

        #[test]
        fn should_limit_the_out_count_to_100() {
            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();

            let friend_key_list: [FriendKey; 200] = [Default::default(); 200];
            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendPresence);
            command.push(friend_key_list.len() as u32);
            command.push_static_buffer(&friend_key_list, 0);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetFriendPresence, 1, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));

            let presense: Vec<FriendPresence> =
                unsafe { result.pop_static_buffer().unwrap().to_vec() };
            assert_eq!(presense, vec![Default::default(); 100]);
        }
    }

    mod get_friend_screen_name {
        use super::*;
        use crate::frd::save::friend_list::FriendEntry;
        use alloc::vec;
        use ctr::sysmodule::server::ServiceContext;

        #[test]
        fn should_return_the_screen_name_and_character_set_for_each_friend_key() {
            let friend_key_1 = FriendKey {
                principal_id: 1,
                ..Default::default()
            };
            let friend_key_2 = FriendKey {
                principal_id: 2,
                ..Default::default()
            };

            let friend_1 = FriendEntry {
                friend_key: friend_key_1,
                friend_relationship: 1,
                ..Default::default()
            };
            let friend_2 = FriendEntry {
                friend_key: friend_key_2,
                friend_relationship: 2,
                ..Default::default()
            };

            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.friend_list.clear();
            context.friend_list.push(friend_1);
            context.friend_list.push(friend_2);

            let friend_key_list = [friend_key_1, friend_key_2];

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendScreenName);
            command.push(friend_key_list.len() as u32);
            command.push(friend_key_list.len() as u32);
            command.push(friend_key_list.len() as u32);
            command.push(0u32);
            command.push(0u32);
            command.push_static_buffer(&friend_key_list, 0);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetFriendScreenName, 1, 4),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));

            let screen_names: Vec<ScreenName> =
                unsafe { result.pop_static_buffer().unwrap().to_vec() };
            assert_eq!(
                screen_names,
                vec![friend_1.screen_name, friend_2.screen_name]
            );

            let character_sets: Vec<TrivialCharacterSet> =
                unsafe { result.pop_static_buffer().unwrap().to_vec() };
            assert_eq!(
                character_sets,
                vec![friend_1.character_set, friend_2.character_set]
            );
        }

        #[test]
        fn should_limit_the_out_count_to_the_number_of_friend_keys() {
            let friend_key_1 = FriendKey {
                principal_id: 1,
                ..Default::default()
            };
            let friend_key_2 = FriendKey {
                principal_id: 2,
                ..Default::default()
            };

            let friend_1 = FriendEntry {
                friend_key: friend_key_1,
                friend_relationship: 1,
                ..Default::default()
            };
            let friend_2 = FriendEntry {
                friend_key: friend_key_2,
                friend_relationship: 2,
                ..Default::default()
            };

            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.friend_list.clear();
            context.friend_list.push(friend_1);
            context.friend_list.push(friend_2);

            let friend_key_list = [friend_key_2];

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendScreenName);
            command.push(2u32);
            command.push(2u32);
            command.push(2u32);
            command.push(0u32);
            command.push(0u32);
            command.push_static_buffer(&friend_key_list, 0);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetFriendScreenName, 1, 4),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));

            let screen_names: Vec<ScreenName> =
                unsafe { result.pop_static_buffer().unwrap().to_vec() };
            assert_eq!(screen_names, vec![friend_2.screen_name]);

            let character_sets: Vec<TrivialCharacterSet> =
                unsafe { result.pop_static_buffer().unwrap().to_vec() };
            assert_eq!(character_sets, vec![friend_2.character_set]);
        }

        #[test]
        fn should_limit_the_out_count_to_the_given_screen_name_out_count() {
            let friend_key_1 = FriendKey {
                principal_id: 1,
                ..Default::default()
            };
            let friend_key_2 = FriendKey {
                principal_id: 2,
                ..Default::default()
            };

            let friend_1 = FriendEntry {
                friend_key: friend_key_1,
                friend_relationship: 1,
                ..Default::default()
            };
            let friend_2 = FriendEntry {
                friend_key: friend_key_2,
                friend_relationship: 2,
                ..Default::default()
            };

            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.friend_list.clear();
            context.friend_list.push(friend_1);
            context.friend_list.push(friend_2);

            let friend_key_list = [friend_key_2, friend_key_1];

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendScreenName);
            command.push(1u32);
            command.push(friend_key_list.len() as u32);
            command.push(friend_key_list.len() as u32);
            command.push(0u32);
            command.push(0u32);
            command.push_static_buffer(&friend_key_list, 0);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetFriendScreenName, 1, 4),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));

            let screen_names: Vec<ScreenName> =
                unsafe { result.pop_static_buffer().unwrap().to_vec() };
            assert_eq!(screen_names, vec![friend_2.screen_name]);

            let character_sets: Vec<TrivialCharacterSet> =
                unsafe { result.pop_static_buffer().unwrap().to_vec() };
            assert_eq!(character_sets, vec![friend_2.character_set]);
        }

        #[test]
        fn should_limit_the_out_count_to_the_given_character_set_out_count() {
            let friend_key_1 = FriendKey {
                principal_id: 1,
                ..Default::default()
            };
            let friend_key_2 = FriendKey {
                principal_id: 2,
                ..Default::default()
            };

            let friend_1 = FriendEntry {
                friend_key: friend_key_1,
                friend_relationship: 1,
                ..Default::default()
            };
            let friend_2 = FriendEntry {
                friend_key: friend_key_2,
                friend_relationship: 2,
                ..Default::default()
            };

            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.friend_list.clear();
            context.friend_list.push(friend_1);
            context.friend_list.push(friend_2);

            let friend_key_list = [friend_key_2, friend_key_1];

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendScreenName);
            command.push(friend_key_list.len() as u32);
            command.push(1u32);
            command.push(friend_key_list.len() as u32);
            command.push(0u32);
            command.push(0u32);
            command.push_static_buffer(&friend_key_list, 0);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetFriendScreenName, 1, 4),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));

            let screen_names: Vec<ScreenName> =
                unsafe { result.pop_static_buffer().unwrap().to_vec() };
            assert_eq!(screen_names, vec![friend_2.screen_name]);

            let character_sets: Vec<TrivialCharacterSet> =
                unsafe { result.pop_static_buffer().unwrap().to_vec() };
            assert_eq!(character_sets, vec![friend_2.character_set]);
        }

        #[test]
        fn should_limit_the_out_count_to_100() {
            let friend_key_1 = FriendKey {
                principal_id: 1,
                ..Default::default()
            };
            let friend_key_2 = FriendKey {
                principal_id: 2,
                ..Default::default()
            };

            let friend_1 = FriendEntry {
                friend_key: friend_key_1,
                friend_relationship: 1,
                ..Default::default()
            };
            let friend_2 = FriendEntry {
                friend_key: friend_key_2,
                friend_relationship: 2,
                ..Default::default()
            };

            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.friend_list.clear();
            context.friend_list.push(friend_1);
            context.friend_list.push(friend_2);

            let friend_key_list: [FriendKey; 200] = [friend_key_1; 200];

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendScreenName);
            command.push(friend_key_list.len() as u32);
            command.push(friend_key_list.len() as u32);
            command.push(friend_key_list.len() as u32);
            command.push(0u32);
            command.push(0u32);
            command.push_static_buffer(&friend_key_list, 0);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetFriendScreenName, 1, 4),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));

            let screen_names: Vec<ScreenName> =
                unsafe { result.pop_static_buffer().unwrap().to_vec() };
            assert_eq!(screen_names.len(), 100);

            let character_sets: Vec<TrivialCharacterSet> =
                unsafe { result.pop_static_buffer().unwrap().to_vec() };
            assert_eq!(character_sets.len(), 100);
        }

        #[test]
        fn should_return_the_default_values_for_friends_that_do_not_exit() {
            let friend_key = FriendKey {
                principal_id: 1,
                ..Default::default()
            };

            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.friend_list.clear();

            let friend_key_list = [friend_key];

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendScreenName);
            command.push(friend_key_list.len() as u32);
            command.push(friend_key_list.len() as u32);
            command.push(friend_key_list.len() as u32);
            command.push(0u32);
            command.push(0u32);
            command.push_static_buffer(&friend_key_list, 0);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetFriendScreenName, 1, 4),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));

            let screen_names: Vec<ScreenName> =
                unsafe { result.pop_static_buffer().unwrap().to_vec() };
            assert_eq!(screen_names, vec![Default::default()]);

            let character_sets: Vec<TrivialCharacterSet> =
                unsafe { result.pop_static_buffer().unwrap().to_vec() };
            assert_eq!(character_sets, vec![Default::default()]);
        }
    }

    mod get_friend_mii {
        use super::*;
        use crate::frd::save::friend_list::FriendEntry;
        use ctr::sysmodule::server::ServiceContext;

        #[test]
        fn should_return_a_mii_for_each_friend() {
            let friend_key_1 = FriendKey {
                principal_id: 1,
                ..Default::default()
            };
            let friend_key_2 = FriendKey {
                principal_id: 2,
                ..Default::default()
            };

            let friend_1 = FriendEntry {
                friend_key: friend_key_1,
                friend_relationship: 1,
                mii: Mii::new([0xff; 96]),
                ..Default::default()
            };
            let friend_2 = FriendEntry {
                friend_key: friend_key_2,
                friend_relationship: 2,
                mii: Mii::new([0xff; 96]),
                ..Default::default()
            };

            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.friend_list.clear();
            context.friend_list.push(friend_1);
            context.friend_list.push(friend_2);

            let friend_key_list = [friend_key_1, friend_key_2];
            let mut out_miis: [Mii; 2] = [Default::default(); 2];

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendMii);
            command.push(friend_key_list.len() as u32);
            command.push_static_buffer(&friend_key_list, 0);
            command.push_write_buffer(&mut out_miis);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetFriendMii, 1, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));

            let miis = unsafe { result.pop_mut_buffer::<Mii>().unwrap() };
            assert_eq!(miis.len(), out_miis.len());
            assert_eq!(miis.as_mut_ptr(), out_miis.as_mut_ptr());
            assert_eq!(out_miis, [Mii::new([0xff; 96]); 2]);
        }

        #[test]
        fn should_limit_the_miis_to_the_provided_max_number() {
            let friend_key_1 = FriendKey {
                principal_id: 1,
                ..Default::default()
            };
            let friend_key_2 = FriendKey {
                principal_id: 2,
                ..Default::default()
            };

            let friend_1 = FriendEntry {
                friend_key: friend_key_1,
                friend_relationship: 1,
                mii: Mii::new([0xff; 96]),
                ..Default::default()
            };
            let friend_2 = FriendEntry {
                friend_key: friend_key_2,
                friend_relationship: 2,
                mii: Mii::new([0xff; 96]),
                ..Default::default()
            };

            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.friend_list.clear();
            context.friend_list.push(friend_1);
            context.friend_list.push(friend_2);

            let friend_key_list = [friend_key_1, friend_key_2];
            let mut out_miis: [Mii; 2] = [Default::default(); 2];

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendMii);
            command.push(1u32);
            command.push_static_buffer(&friend_key_list, 0);
            command.push_write_buffer(&mut out_miis);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetFriendMii, 1, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));

            let miis = unsafe { result.pop_mut_buffer::<Mii>().unwrap() };
            assert_eq!(miis.len(), 1);
            assert_eq!(miis.as_mut_ptr(), out_miis.as_mut_ptr());
            assert_eq!(out_miis, [Mii::new([0xff; 96]), Default::default()])
        }

        #[test]
        fn should_limit_the_miis_to_the_out_buffer_size() {
            let friend_key_1 = FriendKey {
                principal_id: 1,
                ..Default::default()
            };
            let friend_key_2 = FriendKey {
                principal_id: 2,
                ..Default::default()
            };

            let friend_1 = FriendEntry {
                friend_key: friend_key_1,
                friend_relationship: 1,
                mii: Mii::new([0xff; 96]),
                ..Default::default()
            };
            let friend_2 = FriendEntry {
                friend_key: friend_key_2,
                friend_relationship: 2,
                mii: Mii::new([0xff; 96]),
                ..Default::default()
            };

            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.friend_list.clear();
            context.friend_list.push(friend_1);
            context.friend_list.push(friend_2);

            let friend_key_list = [friend_key_1, friend_key_2];
            let mut out_miis: [Mii; 1] = [Default::default()];

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendMii);
            command.push(friend_key_list.len() as u32);
            command.push_static_buffer(&friend_key_list, 0);
            command.push_write_buffer(&mut out_miis);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetFriendMii, 1, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));

            let miis = unsafe { result.pop_mut_buffer::<Mii>().unwrap() };
            assert_eq!(miis.len(), 1);
            assert_eq!(miis.as_mut_ptr(), out_miis.as_mut_ptr());
            assert_eq!(out_miis, [Mii::new([0xff; 96])]);
        }

        #[test]
        fn should_limit_the_out_count_to_100() {
            let friend_key = FriendKey {
                principal_id: 1,
                ..Default::default()
            };

            let friend = FriendEntry {
                friend_key,
                friend_relationship: 1,
                mii: Mii::new([0xff; 96]),
                ..Default::default()
            };

            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.friend_list.clear();
            context.friend_list.push(friend);

            let friend_key_list: [FriendKey; 200] = [Default::default(); 200];
            let mut out_miis: [Mii; 200] = [Default::default(); 200];

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendMii);
            command.push(200u32);
            command.push_static_buffer(&friend_key_list, 0);
            command.push_write_buffer(&mut out_miis);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetFriendMii, 1, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));

            let miis = unsafe { result.pop_mut_buffer::<Mii>().unwrap() };
            assert_eq!(miis.len(), 100);
            assert_eq!(miis.as_mut_ptr(), out_miis.as_mut_ptr());
            assert_eq!(out_miis, [Default::default(); 200]);
        }

        #[test]
        fn should_return_the_default_values_for_friends_that_do_not_exit() {
            let friend_key_1 = FriendKey {
                principal_id: 1,
                ..Default::default()
            };
            let friend_key_2 = FriendKey {
                principal_id: 2,
                ..Default::default()
            };

            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.friend_list.clear();

            let friend_key_list = [friend_key_1, friend_key_2];
            let mut out_miis: [Mii; 2] = [Default::default(); 2];

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendMii);
            command.push(1u32);
            command.push_static_buffer(&friend_key_list, 0);
            command.push_write_buffer(&mut out_miis);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetFriendMii, 1, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));

            let miis = unsafe { result.pop_mut_buffer::<Mii>().unwrap() };
            assert_eq!(miis.len(), 1);
            assert_eq!(miis.as_mut_ptr(), out_miis.as_mut_ptr());
            assert_eq!(out_miis, [Default::default(); 2]);
        }
    }

    mod get_friend_profile {
        use super::*;
        use crate::frd::save::friend_list::FriendEntry;
        use alloc::vec;
        use ctr::sysmodule::server::ServiceContext;

        #[test]
        fn should_return_a_profile_for_each_friend_key() {
            let friend_key_1 = FriendKey {
                principal_id: 1,
                ..Default::default()
            };
            let friend_key_2 = FriendKey {
                principal_id: 2,
                ..Default::default()
            };

            let friend_1 = FriendEntry {
                friend_key: friend_key_1,
                friend_relationship: 1,
                friend_profile: FriendProfile {
                    region: 1,
                    country: 2,
                    area: 3,
                    language: 4,
                    platform: 3,
                    ..Default::default()
                },
                ..Default::default()
            };
            let friend_2 = FriendEntry {
                friend_key: friend_key_2,
                friend_relationship: 2,
                friend_profile: FriendProfile {
                    region: 5,
                    country: 4,
                    area: 3,
                    language: 2,
                    platform: 3,
                    ..Default::default()
                },
                ..Default::default()
            };

            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.friend_list.clear();
            context.friend_list.push(friend_1);
            context.friend_list.push(friend_2);

            let friend_key_list = [friend_key_1, friend_key_2];
            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendProfile);
            command.push(friend_key_list.len() as u32);
            command.push_static_buffer(&friend_key_list, 0);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetFriendProfile, 1, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));

            let profiles: Vec<FriendProfile> =
                unsafe { result.pop_static_buffer().unwrap().to_vec() };
            assert_eq!(
                profiles,
                vec![friend_1.friend_profile, friend_2.friend_profile]
            );
        }

        #[test]
        fn should_limit_the_out_count_to_the_number_of_friend_keys() {
            let friend_key_1 = FriendKey {
                principal_id: 1,
                ..Default::default()
            };
            let friend_key_2 = FriendKey {
                principal_id: 2,
                ..Default::default()
            };

            let friend_1 = FriendEntry {
                friend_key: friend_key_1,
                friend_relationship: 1,
                friend_profile: FriendProfile {
                    region: 1,
                    country: 2,
                    area: 3,
                    language: 4,
                    platform: 3,
                    ..Default::default()
                },
                ..Default::default()
            };
            let friend_2 = FriendEntry {
                friend_key: friend_key_2,
                friend_relationship: 2,
                friend_profile: FriendProfile {
                    region: 5,
                    country: 4,
                    area: 3,
                    language: 2,
                    platform: 3,
                    ..Default::default()
                },
                ..Default::default()
            };

            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.friend_list.clear();
            context.friend_list.push(friend_1);
            context.friend_list.push(friend_2);

            let friend_key_list = [friend_key_1, friend_key_2];
            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendProfile);
            command.push(20u32);
            command.push_static_buffer(&friend_key_list, 0);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetFriendProfile, 1, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));

            let profiles: Vec<FriendProfile> =
                unsafe { result.pop_static_buffer().unwrap().to_vec() };
            assert_eq!(
                profiles,
                vec![friend_1.friend_profile, friend_2.friend_profile]
            );
        }

        #[test]
        fn should_limit_the_out_count_to_the_given_out_count() {
            let friend_key_1 = FriendKey {
                principal_id: 1,
                ..Default::default()
            };
            let friend_key_2 = FriendKey {
                principal_id: 2,
                ..Default::default()
            };

            let friend_1 = FriendEntry {
                friend_key: friend_key_1,
                friend_relationship: 1,
                friend_profile: FriendProfile {
                    region: 1,
                    country: 2,
                    area: 3,
                    language: 4,
                    platform: 3,
                    ..Default::default()
                },
                ..Default::default()
            };
            let friend_2 = FriendEntry {
                friend_key: friend_key_2,
                friend_relationship: 2,
                friend_profile: FriendProfile {
                    region: 5,
                    country: 4,
                    area: 3,
                    language: 2,
                    platform: 3,
                    ..Default::default()
                },
                ..Default::default()
            };

            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.friend_list.clear();
            context.friend_list.push(friend_1);
            context.friend_list.push(friend_2);

            let friend_key_list = [friend_key_1, friend_key_2];
            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendProfile);
            command.push(1u32);
            command.push_static_buffer(&friend_key_list, 0);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetFriendProfile, 1, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));

            let profiles: Vec<FriendProfile> =
                unsafe { result.pop_static_buffer().unwrap().to_vec() };
            assert_eq!(profiles, vec![friend_1.friend_profile]);
        }

        #[test]
        fn should_return_the_default_for_friends_that_do_not_exit() {
            let friend_key = FriendKey {
                principal_id: 1,
                ..Default::default()
            };

            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.friend_list.clear();

            let friend_key_list = [friend_key];
            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendProfile);
            command.push(friend_key_list.len() as u32);
            command.push_static_buffer(&friend_key_list, 0);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetFriendProfile, 1, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));

            let profiles: Vec<FriendProfile> =
                unsafe { result.pop_static_buffer().unwrap().to_vec() };
            assert_eq!(profiles, vec![Default::default()]);
        }

        #[test]
        fn should_limit_the_out_count_to_100() {
            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();

            let friend_key_list: [FriendKey; 200] = [Default::default(); 200];
            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendProfile);
            command.push(friend_key_list.len() as u32);
            command.push_static_buffer(&friend_key_list, 0);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetFriendProfile, 1, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));

            let profiles: Vec<FriendProfile> =
                unsafe { result.pop_static_buffer().unwrap().to_vec() };
            assert_eq!(profiles, vec![Default::default(); 100]);
        }
    }

    mod get_friend_relationship {
        use super::*;
        use crate::frd::save::friend_list::FriendEntry;
        use alloc::vec;
        use ctr::sysmodule::server::ServiceContext;

        #[test]
        fn should_return_friend_relationships_for_each_friend() {
            let friend_key_1 = FriendKey {
                principal_id: 1,
                ..Default::default()
            };
            let friend_key_2 = FriendKey {
                principal_id: 2,
                ..Default::default()
            };

            let friend_1 = FriendEntry {
                friend_key: friend_key_1,
                friend_relationship: 1,
                ..Default::default()
            };
            let friend_2 = FriendEntry {
                friend_key: friend_key_2,
                friend_relationship: 2,
                ..Default::default()
            };

            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.friend_list.clear();
            context.friend_list.push(friend_1);
            context.friend_list.push(friend_2);

            let friend_key_list = [friend_key_1, friend_key_2];
            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendRelationship);
            command.push(friend_key_list.len() as u32);
            command.push_static_buffer(&friend_key_list, 0);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetFriendRelationship, 1, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));

            let friend_relationship: Vec<u8> =
                unsafe { result.pop_static_buffer().unwrap().to_vec() };
            assert_eq!(friend_relationship, vec![1, 2]);
        }

        #[test]
        fn should_fail_if_the_header_has_an_incorrect_normal_param_count() {
            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.friend_list.clear();

            let friend_key_list: [FriendKey; 0] = [];

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendRelationship);
            command.push(0u32);
            command.push(2u32);
            command.push_static_buffer(&friend_key_list, 0);

            let result = handle_frdu_request(&mut context, command.build().into(), 0).unwrap_err();

            assert_eq!(result, GenericResultCode::InvalidCommand.into_result_code())
        }

        #[test]
        fn should_fail_if_the_header_has_an_incorrect_translate_param_count() {
            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.friend_list.clear();

            let friend_key_list: [FriendKey; 0] = [];

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendRelationship);
            command.push(friend_key_list.len() as u32);
            command.push_curent_process_id();
            command.push_curent_process_id();

            let result = handle_frdu_request(&mut context, command.build().into(), 0).unwrap_err();

            assert_eq!(result, GenericResultCode::InvalidCommand.into_result_code())
        }

        #[test]
        fn should_fail_if_the_friend_key_buffer_id_is_not_0() {
            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.friend_list.clear();

            let friend_key_list: [FriendKey; 0] = [];

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendRelationship);
            command.push(friend_key_list.len() as u32);
            command.push_static_buffer(&friend_key_list, 1);

            let result = handle_frdu_request(&mut context, command.build().into(), 0).unwrap_err();

            assert_eq!(result, GenericResultCode::InvalidCommand.into_result_code())
        }

        #[test]
        fn should_limit_the_number_of_results_to_the_number_of_friend_keys() {
            let friend_key_1 = FriendKey {
                principal_id: 1,
                ..Default::default()
            };
            let friend_key_2 = FriendKey {
                principal_id: 2,
                ..Default::default()
            };

            let friend_1 = FriendEntry {
                friend_key: friend_key_1,
                friend_relationship: 1,
                ..Default::default()
            };
            let friend_2 = FriendEntry {
                friend_key: friend_key_2,
                friend_relationship: 2,
                ..Default::default()
            };

            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.friend_list.clear();
            context.friend_list.push(friend_1);
            context.friend_list.push(friend_2);

            let friend_key_list = [friend_key_2];
            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendRelationship);
            command.push(friend_key_list.len() as u32);
            command.push_static_buffer(&friend_key_list, 0);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetFriendRelationship, 1, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));

            let friend_relationship: Vec<u8> =
                unsafe { result.pop_static_buffer().unwrap().to_vec() };
            assert_eq!(friend_relationship, vec![2]);
        }

        #[test]
        fn should_return_0_for_any_friends_that_are_not_found() {
            let friend_key_1 = FriendKey {
                principal_id: 1,
                ..Default::default()
            };
            let friend_key_2 = FriendKey {
                principal_id: 2,
                ..Default::default()
            };

            let friend_1 = FriendEntry {
                friend_key: friend_key_1,
                friend_relationship: 1,
                ..Default::default()
            };

            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.friend_list.clear();
            context.friend_list.push(friend_1);

            let friend_key_list = [friend_key_1, friend_key_2];
            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendRelationship);
            command.push(friend_key_list.len() as u32);
            command.push_static_buffer(&friend_key_list, 0);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetFriendRelationship, 1, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));

            let friend_relationship: Vec<u8> =
                unsafe { result.pop_static_buffer().unwrap().to_vec() };
            assert_eq!(friend_relationship, vec![1, 0]);
        }

        #[test]
        fn should_limit_to_100_friends_max() {
            let friend_key_1 = FriendKey {
                principal_id: 1,
                ..Default::default()
            };
            let friend_1 = FriendEntry {
                friend_key: friend_key_1,
                friend_relationship: 1,
                ..Default::default()
            };

            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.friend_list.clear();
            context.friend_list.push(friend_1);

            let friend_key_list: [FriendKey; 200] = [friend_key_1; 200];
            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendRelationship);
            command.push(friend_key_list.len() as u32);
            command.push_static_buffer(&friend_key_list, 0);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetFriendRelationship, 1, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));

            let friend_relationship: Vec<u8> =
                unsafe { result.pop_static_buffer().unwrap().to_vec() };
            assert_eq!(friend_relationship, vec![1; 100]);
        }
    }

    mod get_friend_attributes {
        use super::*;
        use crate::frd::save::friend_list::FriendEntry;
        use alloc::vec;
        use ctr::sysmodule::server::ServiceContext;

        #[test]
        fn should_return_friend_attributes_for_each_friend() {
            let friend_key_1 = FriendKey {
                principal_id: 1,
                ..Default::default()
            };
            let friend_key_2 = FriendKey {
                principal_id: 2,
                ..Default::default()
            };

            let friend_1 = FriendEntry {
                friend_key: friend_key_1,
                friend_relationship: 1,
                ..Default::default()
            };
            let friend_2 = FriendEntry {
                friend_key: friend_key_2,
                friend_relationship: 2,
                ..Default::default()
            };

            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.friend_list.clear();
            context.friend_list.push(friend_1);
            context.friend_list.push(friend_2);

            let friend_key_list = [friend_key_1, friend_key_2];
            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendAttributeFlags);
            command.push(friend_key_list.len() as u32);
            command.push_static_buffer(&friend_key_list, 0);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetFriendAttributeFlags, 1, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));

            let friend_attributes: Vec<u32> =
                unsafe { result.pop_static_buffer().unwrap().to_vec() };
            assert_eq!(friend_attributes, vec![3, 0]);
        }

        #[test]
        fn should_fail_if_the_header_has_an_incorrect_normal_param_count() {
            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.friend_list.clear();

            let friend_key_list: [FriendKey; 0] = [];

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendAttributeFlags);
            command.push(0u32);
            command.push(2u32);
            command.push_static_buffer(&friend_key_list, 0);

            let result = handle_frdu_request(&mut context, command.build().into(), 0).unwrap_err();

            assert_eq!(result, GenericResultCode::InvalidCommand.into_result_code());
        }

        #[test]
        fn should_fail_if_the_header_has_an_incorrect_translate_param_count() {
            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.friend_list.clear();

            let friend_key_list: [FriendKey; 0] = [];

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendAttributeFlags);
            command.push(friend_key_list.len() as u32);
            command.push_curent_process_id();
            command.push_curent_process_id();

            let result = handle_frdu_request(&mut context, command.build().into(), 0).unwrap_err();

            assert_eq!(result, GenericResultCode::InvalidCommand.into_result_code());
        }

        #[test]
        fn should_fail_if_the_friend_key_buffer_id_is_not_0() {
            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.friend_list.clear();

            let friend_key_list: [FriendKey; 0] = [];

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendAttributeFlags);
            command.push(friend_key_list.len() as u32);
            command.push_static_buffer(&friend_key_list, 1);

            let result = handle_frdu_request(&mut context, command.build().into(), 0).unwrap_err();

            assert_eq!(result, GenericResultCode::InvalidCommand.into_result_code());
        }

        #[test]
        fn should_limit_the_nuber_of_result_to_the_number_of_friend_keys() {
            let friend_key_1 = FriendKey {
                principal_id: 1,
                ..Default::default()
            };
            let friend_key_2 = FriendKey {
                principal_id: 2,
                ..Default::default()
            };

            let friend_1 = FriendEntry {
                friend_key: friend_key_1,
                friend_relationship: 1,
                ..Default::default()
            };
            let friend_2 = FriendEntry {
                friend_key: friend_key_2,
                friend_relationship: 2,
                ..Default::default()
            };

            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.friend_list.clear();
            context.friend_list.push(friend_1);
            context.friend_list.push(friend_2);

            let friend_key_list = [friend_key_2];
            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendAttributeFlags);
            command.push(friend_key_list.len() as u32);
            command.push_static_buffer(&friend_key_list, 0);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetFriendAttributeFlags, 1, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));

            let friend_attributes: Vec<u32> =
                unsafe { result.pop_static_buffer().unwrap().to_vec() };
            assert_eq!(friend_attributes, vec![0]);
        }

        #[test]
        fn should_return_0_for_any_friends_that_are_not_found() {
            let friend_key_1 = FriendKey {
                principal_id: 1,
                ..Default::default()
            };
            let friend_key_2 = FriendKey {
                principal_id: 2,
                ..Default::default()
            };

            let friend_1 = FriendEntry {
                friend_key: friend_key_1,
                friend_relationship: 1,
                ..Default::default()
            };

            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.friend_list.clear();
            context.friend_list.push(friend_1);

            let friend_key_list = [friend_key_1, friend_key_2];
            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendAttributeFlags);
            command.push(friend_key_list.len() as u32);
            command.push_static_buffer(&friend_key_list, 0);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetFriendAttributeFlags, 1, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));

            let friend_attributes: Vec<u32> =
                unsafe { result.pop_static_buffer().unwrap().to_vec() };
            assert_eq!(friend_attributes, vec![3, 0]);
        }

        #[test]
        fn should_limit_to_100_friends_max() {
            let friend_key_1 = FriendKey {
                principal_id: 1,
                ..Default::default()
            };
            let friend_1 = FriendEntry {
                friend_key: friend_key_1,
                friend_relationship: 1,
                ..Default::default()
            };

            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.friend_list.clear();
            context.friend_list.push(friend_1);

            let friend_key_list: [FriendKey; 200] = [friend_key_1; 200];
            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendAttributeFlags);
            command.push(friend_key_list.len() as u32);
            command.push_static_buffer(&friend_key_list, 0);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetFriendAttributeFlags, 1, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));

            let friend_attributes: Vec<u32> =
                unsafe { result.pop_static_buffer().unwrap().to_vec() };
            assert_eq!(friend_attributes, vec![3; 100]);
        }

        #[test]
        fn should_convert_the_friend_relationship_to_attribute_correctly_for_each_attribute() {
            let friend_key_1 = FriendKey {
                principal_id: 1,
                ..Default::default()
            };
            let friend_key_2 = FriendKey {
                principal_id: 2,
                ..Default::default()
            };
            let friend_key_3 = FriendKey {
                principal_id: 3,
                ..Default::default()
            };
            let friend_key_4 = FriendKey {
                principal_id: 4,
                ..Default::default()
            };
            let friend_key_5 = FriendKey {
                principal_id: 5,
                ..Default::default()
            };
            let friend_key_6 = FriendKey {
                principal_id: 6,
                ..Default::default()
            };

            let friend_1 = FriendEntry {
                friend_key: friend_key_1,
                friend_relationship: 0,
                ..Default::default()
            };
            let friend_2 = FriendEntry {
                friend_key: friend_key_2,
                friend_relationship: 1,
                ..Default::default()
            };
            let friend_3 = FriendEntry {
                friend_key: friend_key_3,
                friend_relationship: 2,
                ..Default::default()
            };
            let friend_4 = FriendEntry {
                friend_key: friend_key_4,
                friend_relationship: 3,
                ..Default::default()
            };
            let friend_5 = FriendEntry {
                friend_key: friend_key_5,
                friend_relationship: 4,
                ..Default::default()
            };
            let friend_6 = FriendEntry {
                friend_key: friend_key_6,
                friend_relationship: 5,
                ..Default::default()
            };

            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.friend_list.clear();
            context.friend_list.push(friend_1);
            context.friend_list.push(friend_2);
            context.friend_list.push(friend_3);
            context.friend_list.push(friend_4);
            context.friend_list.push(friend_5);
            context.friend_list.push(friend_6);

            let friend_key_list = [
                friend_key_1,
                friend_key_2,
                friend_key_3,
                friend_key_4,
                friend_key_5,
                friend_key_6,
            ];
            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendAttributeFlags);
            command.push(friend_key_list.len() as u32);
            command.push_static_buffer(&friend_key_list, 0);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetFriendAttributeFlags, 1, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));

            let friend_attributes: Vec<u32> =
                unsafe { result.pop_static_buffer().unwrap().to_vec() };
            assert_eq!(friend_attributes, vec![0, 3, 0, 1, 1, 0]);
        }
    }

    mod get_friend_playing_game {
        use super::*;
        use ctr::sysmodule::server::ServiceContext;

        #[test]
        fn should_return_the_default_game_key_for_each_friend_key() {
            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();

            let friend_key_list: [FriendKey; 2] = [Default::default(); 2];
            let mut out_games: [GameKey; 2] = [Default::default(); 2];

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendPlayingGame);
            command.push(friend_key_list.len() as u32);
            command.push_static_buffer(&friend_key_list, 0);
            command.push_write_buffer(&mut out_games);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetFriendPlayingGame, 1, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));

            let game_keys = unsafe { result.pop_mut_buffer::<GameKey>().unwrap() };
            assert_eq!(game_keys.len(), out_games.len());
            assert_eq!(game_keys.as_mut_ptr(), out_games.as_mut_ptr());
            assert_eq!(out_games, [Default::default(); 2]);
        }

        #[test]
        fn should_limit_the_out_count_to_the_number_of_friend_keys() {
            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();

            let friend_key_list: [FriendKey; 2] = [Default::default(); 2];
            let mut out_games: [GameKey; 2] = [Default::default(); 2];

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendPlayingGame);
            command.push(20u32);
            command.push_static_buffer(&friend_key_list, 0);
            command.push_write_buffer(&mut out_games);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetFriendPlayingGame, 1, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));

            let game_keys = unsafe { result.pop_mut_buffer::<GameKey>().unwrap() };
            assert_eq!(game_keys.len(), out_games.len());
            assert_eq!(game_keys.as_mut_ptr(), out_games.as_mut_ptr());
            assert_eq!(out_games, [Default::default(); 2]);
        }

        #[test]
        fn should_limit_the_out_count_to_the_given_out_count() {
            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();

            let friend_key_list: [FriendKey; 2] = [Default::default(); 2];
            let mut out_games: [GameKey; 2] = [Default::default(); 2];

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendPlayingGame);
            command.push(1u32);
            command.push_static_buffer(&friend_key_list, 0);
            command.push_write_buffer(&mut out_games);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetFriendPlayingGame, 1, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));

            let game_keys = unsafe { result.pop_mut_buffer::<GameKey>().unwrap() };
            assert_eq!(game_keys.len(), 1);
            assert_eq!(game_keys.as_mut_ptr(), out_games.as_mut_ptr());
            assert_eq!(out_games, [Default::default(); 2]);
        }

        #[test]
        fn should_limit_the_out_count_to_the_out_buffer_size() {
            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();

            let friend_key_list: [FriendKey; 2] = [Default::default(); 2];
            let mut out_games: [GameKey; 1] = [Default::default(); 1];

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendPlayingGame);
            command.push(friend_key_list.len() as u32);
            command.push_static_buffer(&friend_key_list, 0);
            command.push_write_buffer(&mut out_games);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetFriendPlayingGame, 1, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));

            let game_keys = unsafe { result.pop_mut_buffer::<GameKey>().unwrap() };
            assert_eq!(game_keys.len(), 1);
            assert_eq!(game_keys.as_mut_ptr(), out_games.as_mut_ptr());
            assert_eq!(out_games, [Default::default(); 1]);
        }

        #[test]
        fn should_limit_the_out_count_to_100() {
            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();

            let friend_key_list: [FriendKey; 200] = [Default::default(); 200];
            let mut out_games: [GameKey; 200] = [Default::default(); 200];

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendPlayingGame);
            command.push(friend_key_list.len() as u32);
            command.push_static_buffer(&friend_key_list, 0);
            command.push_write_buffer(&mut out_games);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetFriendPlayingGame, 1, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));

            let game_keys = unsafe { result.pop_mut_buffer::<GameKey>().unwrap() };
            assert_eq!(game_keys.len(), 100);
            assert_eq!(game_keys.as_mut_ptr(), out_games.as_mut_ptr());
            assert_eq!(out_games, [Default::default(); 200]);
        }
    }

    mod get_friend_favorite_game {
        use super::*;
        use crate::frd::save::friend_list::FriendEntry;
        use alloc::vec;
        use ctr::sysmodule::server::ServiceContext;

        #[test]
        fn should_return_a_favorite_game_for_each_friend_key() {
            let friend_key_1 = FriendKey {
                principal_id: 1,
                ..Default::default()
            };
            let friend_key_2 = FriendKey {
                principal_id: 2,
                ..Default::default()
            };

            let friend_1 = FriendEntry {
                friend_key: friend_key_1,
                friend_relationship: 1,
                favorite_game: GameKey {
                    version: 1,
                    ..Default::default()
                },
                ..Default::default()
            };
            let friend_2 = FriendEntry {
                friend_key: friend_key_2,
                friend_relationship: 2,
                favorite_game: GameKey {
                    version: 2,
                    ..Default::default()
                },
                ..Default::default()
            };

            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.friend_list.clear();
            context.friend_list.push(friend_1);
            context.friend_list.push(friend_2);

            let friend_key_list = [friend_key_1, friend_key_2];
            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendFavoriteGame);
            command.push(friend_key_list.len() as u32);
            command.push_static_buffer(&friend_key_list, 0);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetFriendFavoriteGame, 1, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));

            let profiles: Vec<GameKey> = unsafe { result.pop_static_buffer().unwrap().to_vec() };
            assert_eq!(
                profiles,
                vec![friend_1.favorite_game, friend_2.favorite_game]
            );
        }

        #[test]
        fn should_limit_the_out_count_to_the_number_of_friend_keys() {
            let friend_key_1 = FriendKey {
                principal_id: 1,
                ..Default::default()
            };
            let friend_key_2 = FriendKey {
                principal_id: 2,
                ..Default::default()
            };

            let friend_1 = FriendEntry {
                friend_key: friend_key_1,
                friend_relationship: 1,
                favorite_game: GameKey {
                    version: 1,
                    ..Default::default()
                },
                ..Default::default()
            };
            let friend_2 = FriendEntry {
                friend_key: friend_key_2,
                friend_relationship: 2,
                favorite_game: GameKey {
                    version: 2,
                    ..Default::default()
                },
                ..Default::default()
            };

            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.friend_list.clear();
            context.friend_list.push(friend_1);
            context.friend_list.push(friend_2);

            let friend_key_list = [friend_key_1, friend_key_2];
            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendFavoriteGame);
            command.push(20u32);
            command.push_static_buffer(&friend_key_list, 0);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetFriendFavoriteGame, 1, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));

            let profiles: Vec<GameKey> = unsafe { result.pop_static_buffer().unwrap().to_vec() };
            assert_eq!(
                profiles,
                vec![friend_1.favorite_game, friend_2.favorite_game]
            );
        }

        #[test]
        fn should_limit_the_out_count_to_the_given_out_count() {
            let friend_key_1 = FriendKey {
                principal_id: 1,
                ..Default::default()
            };
            let friend_key_2 = FriendKey {
                principal_id: 2,
                ..Default::default()
            };

            let friend_1 = FriendEntry {
                friend_key: friend_key_1,
                friend_relationship: 1,
                favorite_game: GameKey {
                    version: 1,
                    ..Default::default()
                },
                ..Default::default()
            };
            let friend_2 = FriendEntry {
                friend_key: friend_key_2,
                friend_relationship: 2,
                favorite_game: GameKey {
                    version: 2,
                    ..Default::default()
                },
                ..Default::default()
            };

            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.friend_list.clear();
            context.friend_list.push(friend_1);
            context.friend_list.push(friend_2);

            let friend_key_list = [friend_key_1, friend_key_2];
            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendFavoriteGame);
            command.push(1u32);
            command.push_static_buffer(&friend_key_list, 0);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetFriendFavoriteGame, 1, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));

            let profiles: Vec<GameKey> = unsafe { result.pop_static_buffer().unwrap().to_vec() };
            assert_eq!(profiles, vec![friend_1.favorite_game]);
        }

        #[test]
        fn should_return_the_default_for_friends_that_do_not_exit() {
            let friend_key = FriendKey {
                principal_id: 1,
                ..Default::default()
            };

            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.friend_list.clear();

            let friend_key_list = [friend_key];
            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendFavoriteGame);
            command.push(friend_key_list.len() as u32);
            command.push_static_buffer(&friend_key_list, 0);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetFriendFavoriteGame, 1, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));

            let profiles: Vec<GameKey> = unsafe { result.pop_static_buffer().unwrap().to_vec() };
            assert_eq!(profiles, vec![Default::default()]);
        }

        #[test]
        fn should_limit_the_out_count_to_100() {
            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();

            let friend_key_list: [FriendKey; 200] = [Default::default(); 200];
            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendFavoriteGame);
            command.push(friend_key_list.len() as u32);
            command.push_static_buffer(&friend_key_list, 0);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetFriendFavoriteGame, 1, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));

            let profiles: Vec<GameKey> = unsafe { result.pop_static_buffer().unwrap().to_vec() };
            assert_eq!(profiles, vec![Default::default(); 100]);
        }
    }

    mod get_friend_info {
        use super::*;
        use crate::frd::save::friend_list::FriendEntry;
        use ctr::{frd::CharacterSet, ipc::BufferRights};

        #[test]
        fn should_store_the_friend_info_in_the_provided_buffer() {
            let mut out: [FriendInfo; 2] = [Default::default(), Default::default()];
            let friend_entry_1: FriendEntry = Default::default();
            let friend_entry_2: FriendEntry = Default::default();

            let friend_keys = [friend_entry_1.friend_key, friend_entry_2.friend_key];

            let mut context = FriendServiceContext::new().unwrap();
            context.friend_list.clear();
            context.friend_list.push(friend_entry_1);
            context.friend_list.push(friend_entry_2);

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendInfo);
            command.push(friend_keys.len() as u32);
            command.push(0u32);
            command.push(CharacterSet::JapanUsaEuropeAustralia as u32);
            command.push_static_buffer(&friend_keys, 0);
            command.push_write_buffer(&mut out);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetFriendInfo, 1, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(
                result.pop_and_validate_buffer(
                    2 * core::mem::size_of::<FriendInfo>(),
                    BufferRights::Write,
                    out.as_ptr() as usize
                ),
                Ok(())
            );

            assert_eq!(
                out,
                [
                    FriendInfo::from(friend_entry_1),
                    FriendInfo::from(friend_entry_2)
                ]
            );
        }

        #[test]
        fn should_only_return_friends_that_are_found() {
            let mut out: [FriendInfo; 2] = [Default::default(), Default::default()];
            let friend_entry_1 = FriendEntry {
                friend_key: FriendKey {
                    principal_id: 1,
                    local_friend_code: 0xaabbccdd,
                    ..Default::default()
                },
                ..Default::default()
            };
            let friend_entry_2 = FriendEntry {
                friend_key: FriendKey {
                    principal_id: 2,
                    local_friend_code: 0x11223344,
                    ..Default::default()
                },
                ..Default::default()
            };

            let friend_keys = [friend_entry_1.friend_key, friend_entry_2.friend_key];

            let mut context = FriendServiceContext::new().unwrap();
            context.friend_list.clear();
            context.friend_list.push(friend_entry_1);

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendInfo);
            command.push(friend_keys.len() as u32);
            command.push(0u32);
            command.push(CharacterSet::JapanUsaEuropeAustralia as u32);
            command.push_static_buffer(&friend_keys, 0);
            command.push_write_buffer(&mut out);
            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetFriendInfo, 1, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(
                result.pop_and_validate_buffer(
                    2 * core::mem::size_of::<FriendInfo>(),
                    BufferRights::Write,
                    out.as_ptr() as usize
                ),
                Ok(())
            );
            assert_eq!(out, [FriendInfo::from(friend_entry_1), Default::default()]);
        }

        #[test]
        fn should_limit_the_out_count_to_the_friend_keys_if_they_are_smaller_than_the_out_buffer() {
            let mut out: [FriendInfo; 2] = [Default::default(), Default::default()];
            let friend_keys = [FriendKey {
                principal_id: 1,
                ..Default::default()
            }];

            let mut context = FriendServiceContext::new().unwrap();
            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendInfo);
            command.push(friend_keys.len() as u32);
            command.push(0u32);
            command.push(CharacterSet::JapanUsaEuropeAustralia as u32);
            command.push_static_buffer(&friend_keys, 0);
            command.push_write_buffer(&mut out);
            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetFriendInfo, 1, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(
                result.pop_and_validate_buffer(
                    core::mem::size_of::<FriendInfo>(),
                    BufferRights::Write,
                    out.as_ptr() as usize
                ),
                Ok(())
            );
        }

        #[test]
        fn should_limit_the_out_count_to_the_out_buffer_if_it_is_smaller_than_the_friend_keys() {
            let mut out: [FriendInfo; 1] = [Default::default()];
            let friend_keys = [
                FriendKey {
                    principal_id: 1,
                    ..Default::default()
                },
                FriendKey {
                    principal_id: 2,
                    ..Default::default()
                },
            ];

            let mut context = FriendServiceContext::new().unwrap();
            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendInfo);
            command.push(friend_keys.len() as u32);
            command.push(0u32);
            command.push(CharacterSet::JapanUsaEuropeAustralia as u32);
            command.push_static_buffer(&friend_keys, 0);
            command.push_write_buffer(&mut out);
            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetFriendInfo, 1, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(
                result.pop_and_validate_buffer(
                    core::mem::size_of::<FriendInfo>(),
                    BufferRights::Write,
                    out.as_ptr() as usize
                ),
                Ok(())
            );
        }

        #[test]
        fn should_fail_if_the_header_has_incorrect_normal_params() {
            let mut out: [FriendInfo; 2] = [Default::default(), Default::default()];
            let friend_keys: [FriendKey; 1] = [Default::default()];

            let mut context = FriendServiceContext::new().unwrap();
            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendInfo);
            command.push(friend_keys.len() as u32);
            command.push(0u32);
            command.push(CharacterSet::JapanUsaEuropeAustralia as u32);
            command.push_static_buffer(&friend_keys, 0);
            command.push_write_buffer(&mut out);

            let mut built_command = command.build();
            // Force command to have valid values, but an incorrect normal param count
            built_command.param_pool[0] = 0x1a0100;

            let result = handle_frdu_request(&mut context, built_command.into(), 0).unwrap_err();

            assert_eq!(result, GenericResultCode::InvalidCommand.into_result_code());
        }

        #[test]
        fn should_fail_if_the_header_has_incorrect_translate_params() {
            let mut out: [FriendInfo; 2] = [Default::default(), Default::default()];
            let friend_keys: [FriendKey; 1] = [Default::default()];

            let mut context = FriendServiceContext::new().unwrap();
            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendInfo);
            command.push(friend_keys.len() as u32);
            command.push(0u32);
            command.push(CharacterSet::JapanUsaEuropeAustralia as u32);
            command.push_static_buffer(&friend_keys, 0);
            command.push_write_buffer(&mut out);

            let mut built_command = command.build();
            // Force command to have valid values, but an incorrect translate param count
            built_command.param_pool[0] = 0x1a00c1;

            let result = handle_frdu_request(&mut context, built_command.into(), 0).unwrap_err();

            assert_eq!(result, GenericResultCode::InvalidCommand.into_result_code());
        }

        #[test]
        fn should_fail_if_the_friend_list_buffer_id_is_not_0() {
            let mut out: [FriendInfo; 2] = [Default::default(), Default::default()];
            let friend_keys: [FriendKey; 1] = [Default::default()];

            let mut context = FriendServiceContext::new().unwrap();
            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendInfo);
            command.push(friend_keys.len() as u32);
            command.push(0u32);
            command.push(CharacterSet::JapanUsaEuropeAustralia as u32);
            command.push_static_buffer(&friend_keys, 1);
            command.push_write_buffer(&mut out);
            let result = handle_frdu_request(&mut context, command.build().into(), 0).unwrap_err();

            assert_eq!(result, GenericResultCode::InvalidCommand.into_result_code());
        }
    }

    mod is_included_in_friends_list {
        use super::*;
        use crate::frd::save::friend_list::FriendEntry;

        #[test]
        fn should_return_true_if_the_friend_is_in_the_friends_list() {
            let friend = FriendEntry {
                friend_key: FriendKey {
                    local_friend_code: 0xAAAABBBBCCCCDDDD,
                    ..Default::default()
                },
                ..Default::default()
            };

            let mut context = FriendServiceContext::new().unwrap();
            context.friend_list.clear();
            context.friend_list.push(friend);

            let mut command = ThreadCommandBuilder::new(FrdUCommand::IsIncludedInFriendList);
            command.push_u64(friend.friend_key.local_friend_code);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::IsIncludedInFriendList, 2, 0),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(result.pop() != 0, true);
        }

        #[test]
        fn should_return_false_if_the_friend_is_not_in_the_friends_list() {
            let mut context = FriendServiceContext::new().unwrap();
            context.friend_list.clear();

            let mut command = ThreadCommandBuilder::new(FrdUCommand::IsIncludedInFriendList);
            command.push_u64(0xAAAABBBBCCCCDDDD);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::IsIncludedInFriendList, 2, 0),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(result.pop() != 0, false);
        }
    }

    mod unscramble_local_friend_code {
        use super::*;
        use crate::frd::save::friend_list::FriendEntry;
        use alloc::vec;
        use ctr::sysmodule::server::ServiceContext;

        #[test]
        fn should_unscramble_the_friend_code_for_all_given_friend_codes() {
            let friend_key_1 = FriendKey {
                local_friend_code: 0xAAAABBBBCCCCDDDD,
                ..Default::default()
            };
            let friend_key_2 = FriendKey {
                local_friend_code: 0x1111222233334444,
                ..Default::default()
            };

            let friend_1 = FriendEntry {
                friend_key: friend_key_1,
                ..Default::default()
            };
            let friend_2 = FriendEntry {
                friend_key: friend_key_2,
                ..Default::default()
            };

            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.friend_list.clear();
            context.friend_list.push(friend_1);
            context.friend_list.push(friend_2);

            let friend_code_list = [
                ScrambledFriendCode::new(friend_key_1.local_friend_code, 0x1234),
                ScrambledFriendCode::new(friend_key_2.local_friend_code, 0xabcd),
            ];
            let mut command = ThreadCommandBuilder::new(FrdUCommand::UnscrambleLocalFriendCode);
            command.push(friend_code_list.len() as u32);
            command.push_static_buffer(&friend_code_list, 1);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::UnscrambleLocalFriendCode, 1, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));

            let friend_codes: Vec<u64> = unsafe { result.pop_static_buffer().unwrap().to_vec() };
            assert_eq!(
                friend_codes,
                vec![
                    friend_key_1.local_friend_code,
                    friend_key_2.local_friend_code
                ]
            );
        }

        #[test]
        fn should_return_0_for_any_friend_code_not_in_the_friends_list() {
            let friend_key_1 = FriendKey {
                local_friend_code: 0xAAAABBBBCCCCDDDD,
                ..Default::default()
            };
            let friend_key_2 = FriendKey {
                local_friend_code: 0x1111222233334444,
                ..Default::default()
            };

            let friend_1 = FriendEntry {
                friend_key: friend_key_1,
                ..Default::default()
            };

            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.friend_list.clear();
            context.friend_list.push(friend_1);

            let friend_code_list = [
                ScrambledFriendCode::new(friend_key_1.local_friend_code, 0x1234),
                ScrambledFriendCode::new(friend_key_2.local_friend_code, 0xabcd),
            ];
            let mut command = ThreadCommandBuilder::new(FrdUCommand::UnscrambleLocalFriendCode);
            command.push(friend_code_list.len() as u32);
            command.push_static_buffer(&friend_code_list, 1);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::UnscrambleLocalFriendCode, 1, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));

            let friend_codes: Vec<u64> = unsafe { result.pop_static_buffer().unwrap().to_vec() };
            assert_eq!(friend_codes, vec![friend_key_1.local_friend_code, 0]);
        }

        #[test]
        fn should_limit_the_output_to_the_given_friend_codes() {
            let friend_key_1 = FriendKey {
                local_friend_code: 0xAAAABBBBCCCCDDDD,
                ..Default::default()
            };
            let friend_key_2 = FriendKey {
                local_friend_code: 0x1111222233334444,
                ..Default::default()
            };

            let friend_1 = FriendEntry {
                friend_key: friend_key_1,
                ..Default::default()
            };
            let friend_2 = FriendEntry {
                friend_key: friend_key_2,
                ..Default::default()
            };

            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.friend_list.clear();
            context.friend_list.push(friend_1);
            context.friend_list.push(friend_2);

            let friend_code_list = [ScrambledFriendCode::new(
                friend_key_1.local_friend_code,
                0x1234,
            )];
            let mut command = ThreadCommandBuilder::new(FrdUCommand::UnscrambleLocalFriendCode);
            command.push(2u32);
            command.push_static_buffer(&friend_code_list, 1);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::UnscrambleLocalFriendCode, 1, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));

            let friend_codes: Vec<u64> = unsafe { result.pop_static_buffer().unwrap().to_vec() };
            assert_eq!(friend_codes, vec![friend_key_1.local_friend_code,]);
        }

        #[test]
        fn should_limit_the_output_to_the_given_max_out_count() {
            let friend_key_1 = FriendKey {
                local_friend_code: 0xAAAABBBBCCCCDDDD,
                ..Default::default()
            };
            let friend_key_2 = FriendKey {
                local_friend_code: 0x1111222233334444,
                ..Default::default()
            };

            let friend_1 = FriendEntry {
                friend_key: friend_key_1,
                ..Default::default()
            };
            let friend_2 = FriendEntry {
                friend_key: friend_key_2,
                ..Default::default()
            };

            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.friend_list.clear();
            context.friend_list.push(friend_1);
            context.friend_list.push(friend_2);

            let friend_code_list = [
                ScrambledFriendCode::new(friend_key_1.local_friend_code, 0x1234),
                ScrambledFriendCode::new(friend_key_2.local_friend_code, 0xabcd),
            ];
            let mut command = ThreadCommandBuilder::new(FrdUCommand::UnscrambleLocalFriendCode);
            command.push(1u32);
            command.push_static_buffer(&friend_code_list, 1);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::UnscrambleLocalFriendCode, 1, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));

            let friend_codes: Vec<u64> = unsafe { result.pop_static_buffer().unwrap().to_vec() };
            assert_eq!(friend_codes, vec![friend_key_1.local_friend_code,]);
        }

        #[test]
        fn should_limit_the_output_to_100() {
            let friend_key = FriendKey {
                local_friend_code: 0xAAAABBBBCCCCDDDD,
                ..Default::default()
            };

            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.friend_list.clear();

            let friend_code_list: [ScrambledFriendCode; 200] =
                [ScrambledFriendCode::new(friend_key.local_friend_code, 0x1234); 200];
            let mut command = ThreadCommandBuilder::new(FrdUCommand::UnscrambleLocalFriendCode);
            command.push(friend_code_list.len() as u32);
            command.push_static_buffer(&friend_code_list, 1);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::UnscrambleLocalFriendCode, 1, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));

            let friend_codes: Vec<u64> = unsafe { result.pop_static_buffer().unwrap().to_vec() };
            assert_eq!(friend_codes, vec![0; 100]);
        }
    }

    mod attach_to_event_notification {
        use super::*;
        use ctr::sysmodule::server::ServiceContext;
        use ctr::Handle;

        #[test]
        fn should_set_the_client_event_handler() {
            let session_index = 0;
            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.session_contexts[session_index].client_event = None;

            let mut command = ThreadCommandBuilder::new(FrdUCommand::AttachToEventNotification);
            command.push_raw_handle(0xabcd);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), session_index)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::AttachToEventNotification, 1, 0),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(
                context.session_contexts[session_index].client_event,
                Some(Handle::from(0xabcdu32))
            );
        }
    }

    mod set_notification_mask {
        use super::*;
        use ctr::sysmodule::server::ServiceContext;

        #[test]
        fn should_set_the_notification_mask() {
            let session_index = 0;
            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.session_contexts[session_index].notification_mask = 0;

            let mut command = ThreadCommandBuilder::new(FrdUCommand::SetNotificationMask);
            command.push(1u32);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), session_index)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::SetNotificationMask, 1, 0),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(context.session_contexts[session_index].notification_mask, 1);
        }
    }

    mod get_event_notification {
        use super::*;
        use ctr::frd::NotificationType;
        use ctr::sysmodule::server::ServiceContext;

        #[test]
        fn should_set_the_notifications_in_the_out_buffer() {
            let notification_1 =
                NotificationEvent::new(NotificationType::FriendWentOnline, Default::default());
            let notification_2 =
                NotificationEvent::new(NotificationType::FriendWentOffline, Default::default());

            let session_index = 0;
            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();

            let client_event_queue =
                &mut context.session_contexts[session_index].client_event_queue;
            client_event_queue.push(notification_1);
            client_event_queue.push(notification_2);

            let mut out_notifications: [NotificationEvent; 2] = [Default::default(); 2];

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetEventNotification);
            command.push(2u32);
            command.push_write_buffer(&mut out_notifications);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetEventNotification, 3, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(result.pop(), 0);
            assert_eq!(result.pop(), 2);

            let notifications = unsafe { result.pop_mut_buffer::<NotificationEvent>().unwrap() };
            assert_eq!(notifications.len(), out_notifications.len());
            assert_eq!(notifications.as_mut_ptr(), out_notifications.as_mut_ptr());
            assert_eq!(out_notifications, [notification_1, notification_2]);
        }

        #[test]
        fn should_limit_the_out_count_to_the_given_max_out_count() {
            let notification_1 =
                NotificationEvent::new(NotificationType::FriendWentOnline, Default::default());
            let notification_2 =
                NotificationEvent::new(NotificationType::FriendWentOffline, Default::default());

            let session_index = 0;
            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();

            let client_event_queue =
                &mut context.session_contexts[session_index].client_event_queue;
            client_event_queue.push(notification_1);
            client_event_queue.push(notification_2);

            let mut out_notifications: [NotificationEvent; 2] = [Default::default(); 2];

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetEventNotification);
            command.push(1u32);
            command.push_write_buffer(&mut out_notifications);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetEventNotification, 3, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(result.pop(), 0);
            assert_eq!(result.pop(), 1);

            let notifications = unsafe { result.pop_mut_buffer::<NotificationEvent>().unwrap() };
            assert_eq!(notifications.len(), 1);
            assert_eq!(notifications.as_mut_ptr(), out_notifications.as_mut_ptr());
            assert_eq!(out_notifications, [notification_1, Default::default()]);
        }

        #[test]
        fn should_limit_the_out_count_to_the_event_queue_size() {
            let notification_1 =
                NotificationEvent::new(NotificationType::FriendWentOnline, Default::default());

            let session_index = 0;
            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();

            let client_event_queue =
                &mut context.session_contexts[session_index].client_event_queue;
            client_event_queue.push(notification_1);

            let mut out_notifications: [NotificationEvent; 2] = [Default::default(); 2];

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetEventNotification);
            command.push(2u32);
            command.push_write_buffer(&mut out_notifications);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetEventNotification, 3, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(result.pop(), 0);
            assert_eq!(result.pop(), 1);

            let notifications = unsafe { result.pop_mut_buffer::<NotificationEvent>().unwrap() };
            assert_eq!(notifications.len(), 1);
            assert_eq!(notifications.as_mut_ptr(), out_notifications.as_mut_ptr());
            assert_eq!(out_notifications, [notification_1, Default::default()]);
        }

        #[test]
        fn should_limit_the_out_count_to_the_buffer_size() {
            let notification_1 =
                NotificationEvent::new(NotificationType::FriendWentOnline, Default::default());
            let notification_2 =
                NotificationEvent::new(NotificationType::FriendWentOffline, Default::default());

            let session_index = 0;
            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();

            let client_event_queue =
                &mut context.session_contexts[session_index].client_event_queue;
            client_event_queue.push(notification_1);
            client_event_queue.push(notification_2);

            let mut out_notifications: [NotificationEvent; 1] = [Default::default()];

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetEventNotification);
            command.push(2u32);
            command.push_write_buffer(&mut out_notifications);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetEventNotification, 3, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(result.pop(), 0);
            assert_eq!(result.pop(), 1);

            let notifications = unsafe { result.pop_mut_buffer::<NotificationEvent>().unwrap() };
            assert_eq!(notifications.len(), 1);
            assert_eq!(notifications.as_mut_ptr(), out_notifications.as_mut_ptr());
            assert_eq!(out_notifications, [notification_1]);
        }

        #[test]
        fn should_clear_the_event_queue_after_running() {
            let notification_1 =
                NotificationEvent::new(NotificationType::FriendWentOnline, Default::default());
            let notification_2 =
                NotificationEvent::new(NotificationType::FriendWentOffline, Default::default());

            let session_index = 0;
            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();

            let client_event_queue =
                &mut context.session_contexts[session_index].client_event_queue;
            client_event_queue.push(notification_1);
            client_event_queue.push(notification_2);

            let mut out_notifications: [NotificationEvent; 2] = [Default::default(); 2];

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetEventNotification);
            command.push(2u32);
            command.push_write_buffer(&mut out_notifications);

            handle_frdu_request(&mut context, command.build().into(), 0).unwrap();

            assert_eq!(
                context.session_contexts[session_index]
                    .client_event_queue
                    .len(),
                0
            );
        }
    }

    mod principal_id_to_friend_code {
        use super::*;
        use crate::frd::result::FrdErrorCode;

        #[test]
        fn should_return_friend_code_if_the_principal_id_is_valid() {
            let mut context = FriendServiceContext::new().unwrap();

            let mut command = ThreadCommandBuilder::new(FrdUCommand::PrincipalIdToFriendCode);
            command.push(0xaabbccddu32);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::PrincipalIdToFriendCode, 3, 0),
                Ok(()),
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(result.pop(), 0xaabbccdd);
            assert_eq!(result.pop(), 0x38);
        }

        #[test]
        fn should_return_error_code_if_the_principal_id_is_zero() {
            let mut context = FriendServiceContext::new().unwrap();

            let mut command = ThreadCommandBuilder::new(FrdUCommand::PrincipalIdToFriendCode);
            command.push(0u32);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::PrincipalIdToFriendCode, 3, 0),
                Ok(()),
            );
            assert_eq!(
                result.pop_result().unwrap_err(),
                FrdErrorCode::InvalidPrincipalId.into_result_code(),
            );
            assert_eq!(result.pop(), 0);
            assert_eq!(result.pop(), 0);
        }
    }

    mod friend_code_to_principal_id {
        use super::*;
        use crate::frd::result::FrdErrorCode;

        #[test]
        fn should_return_the_principal_id_if_the_friend_code_is_valid() {
            let mut context = FriendServiceContext::new().unwrap();

            let mut command = ThreadCommandBuilder::new(FrdUCommand::FriendCodeToPrincipalId);
            command.push_u64(0x38aabbccdd);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::FriendCodeToPrincipalId, 2, 0),
                Ok(()),
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(result.pop(), 0xaabbccdd);
        }

        #[test]
        fn should_return_an_error_code_if_the_friend_code_is_invalid() {
            let mut context = FriendServiceContext::new().unwrap();

            let mut command = ThreadCommandBuilder::new(FrdUCommand::FriendCodeToPrincipalId);
            command.push_u64(0x40aabbccdd);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::FriendCodeToPrincipalId, 2, 0),
                Ok(()),
            );
            assert_eq!(
                result.pop_result().unwrap_err(),
                FrdErrorCode::InvalidFriendCode.into_result_code(),
            );
            assert_eq!(result.pop(), 0);
        }
    }

    mod is_valid_friend_code {
        use super::*;

        #[test]
        fn should_return_true_if_valid_friend_code() {
            let mut context = FriendServiceContext::new().unwrap();

            let mut command = ThreadCommandBuilder::new(FrdUCommand::IsValidFriendCode);
            command.push_u64(0x38aabbccdd);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::IsValidFriendCode, 2, 0),
                Ok(()),
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(result.pop(), 1);
        }

        #[test]
        fn should_return_false_if_invalid_friend_code() {
            let mut context = FriendServiceContext::new().unwrap();

            let mut command = ThreadCommandBuilder::new(FrdUCommand::IsValidFriendCode);
            command.push_u64(0x40aabbccdd);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::IsValidFriendCode, 2, 0),
                Ok(()),
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(result.pop(), 0);
        }
    }

    mod result_to_error_code {
        use super::*;

        #[test]
        fn should_return_an_error_and_0_if_bits_10_through_18_are_not_0x31() {
            let mut context = FriendServiceContext::new().unwrap();
            let mut command = ThreadCommandBuilder::new(FrdUCommand::ResultToErrorCode);
            command.push(0b11111111111111111111111111110000u32);
            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::ResultToErrorCode, 2, 0),
                Ok(())
            );
            assert_eq!(
                result.pop_result().unwrap_err(),
                FrdErrorCode::InvalidErrorCode.into_result_code()
            );
            assert_eq!(result.pop(), 0);
        }

        #[test]
        fn should_return_a_success_and_0_if_bits_10_through_18_are_0x31_and_the_result_code_is_greater_than_negative_1(
        ) {
            let mut context = FriendServiceContext::new().unwrap();
            let mut command = ThreadCommandBuilder::new(FrdUCommand::ResultToErrorCode);
            command.push(0xc400u32);
            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::ResultToErrorCode, 2, 0),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(result.pop(), 0);
        }

        #[test]
        fn should_return_a_success_and_0x59d8_if_bits_10_through_18_are_0x31_and_the_result_code_is_less_than_0_and_bits_0_through_10_are_0x101(
        ) {
            let mut context = FriendServiceContext::new().unwrap();
            let mut command = ThreadCommandBuilder::new(FrdUCommand::ResultToErrorCode);
            command.push(-0x33affi32 as u32);
            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::ResultToErrorCode, 2, 0),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(result.pop(), 0x59d8);
        }

        #[test]
        fn should_return_a_success_and_0x2710_if_bits_10_through_18_are_0x31_and_the_result_code_is_less_than_0_and_bits_0_through_10_are_not_0x101(
        ) {
            let mut context = FriendServiceContext::new().unwrap();
            let mut command = ThreadCommandBuilder::new(FrdUCommand::ResultToErrorCode);
            command.push(-0x33af0i32 as u32);
            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::ResultToErrorCode, 2, 0),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(result.pop(), 0x2710);
        }
    }

    mod request_game_authentication {
        use super::*;
        use ctr::http::{HttpContext, RequestMethod};
        use ctr::sysmodule::server::ServiceContext;
        use safe_transmute::transmute_to_bytes_mut;

        #[test]
        fn should_send_an_authentication_request_made_from_the_given_parameters() {
            let auth_response = "locator=MTI3LjAuMC4xOjcwMDA*&retry=MA**&returncd=MDAx&token=AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8gISIjJCUmJygpKissLS4vMDE*.AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8gISIjJCUmJygpKissLS4vMDE*&datetime=MjAyMTAxMDIwMzA0MDU*".clone();

            let downloaded_auth_response = auth_response.clone();
            HttpContext::download_data_into_buffer.mock_safe(move |_, buffer| {
                let response_bytes = downloaded_auth_response.as_bytes();
                buffer[..response_bytes.len()].clone_from_slice(response_bytes);
                MockResult::Return(Ok(()))
            });

            create_game_login_request.mock_safe(
                |_,
                 requesting_process_id,
                 requesting_game_id,
                 sdk_version_low,
                 sdk_version_high,
                 ingamesn| {
                    assert_eq!(requesting_process_id, 0);
                    assert_eq!(requesting_game_id, 1234);
                    assert_eq!(sdk_version_low, 3);
                    assert_eq!(sdk_version_high, 4);
                    assert_eq!(ingamesn, "ingamesn-test");
                    MockResult::Return(HttpContext::new("", RequestMethod::Post))
                },
            );

            let mut ingamesn: [u32; 6] = [0; 6];
            let ingamesn_buffer = transmute_to_bytes_mut(&mut ingamesn);
            ingamesn_buffer[..13].copy_from_slice("ingamesn-test".as_bytes());

            let session_index = 0usize;
            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.session_contexts[session_index].last_service_locator_response = None;

            let mut command = ThreadCommandBuilder::new(FrdUCommand::RequestGameAuthentication);
            command.push(1234u32);
            command.push_struct(&ingamesn);
            command.push(3u32);
            command.push(4u32);
            command.push_curent_process_id();
            command.push_raw_handle(5678u32);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), session_index)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::RequestGameAuthentication, 1, 0),
                Ok(()),
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(
                context.session_contexts[session_index]
                    .last_game_authentication_response
                    .unwrap(),
                GameAuthenticationData::from_fetched_response(auth_response, 200).unwrap()
            );
        }

        #[test]
        fn should_ignore_ingamesn_bytes_after_null_terminator() {
            create_game_login_request.mock_safe(
                |_,
                 _requesting_process_id,
                 _requesting_game_id,
                 _sdk_version_low,
                 _sdk_version_high,
                 ingamesn| {
                    assert_eq!(ingamesn, "");
                    MockResult::Return(HttpContext::new("", RequestMethod::Post))
                },
            );

            let mut ingamesn: [u32; 6] = [0; 6];
            let ingamesn_buffer = transmute_to_bytes_mut(&mut ingamesn);
            ingamesn_buffer[1..14].copy_from_slice("ingamesn_test".as_bytes());

            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();

            let mut command = ThreadCommandBuilder::new(FrdUCommand::RequestGameAuthentication);
            command.push(1234u32);
            command.push_struct(&ingamesn);
            command.push(3u32);
            command.push(4u32);
            command.push_curent_process_id();
            command.push_raw_handle(5678u32);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::RequestGameAuthentication, 1, 0),
                Ok(()),
            );
            assert_eq!(result.pop_result(), Ok(()));
        }
    }

    mod get_game_authentication_data {
        use super::*;
        use crate::frd::online_play::authentication::GameAuthenticationData;
        use alloc::vec;
        use ctr::sysmodule::server::ServiceContext;

        #[test]
        fn should_return_success_if_auth_data_exists() {
            let session_index = 0usize;
            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.session_contexts[session_index].last_game_authentication_response =
                Some(Default::default());

            let command = ThreadCommandBuilder::new(FrdUCommand::GetGameAuthenticationData);
            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), session_index)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetGameAuthenticationData, 1, 2),
                Ok(()),
            );
            assert_eq!(result.validate_buffer_id(2, 0), Ok(()));
            assert_eq!(result.pop_result(), Ok(()));
            let game_auth_data: Vec<GameAuthenticationData> =
                unsafe { result.pop_static_buffer().unwrap().to_vec() };
            assert_eq!(game_auth_data, vec![Default::default()])
        }

        #[test]
        fn should_return_error_if_auth_data_does_not_exist() {
            let session_index = 0usize;
            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.session_contexts[session_index].last_service_locator_response = None;

            let command = ThreadCommandBuilder::new(FrdUCommand::GetGameAuthenticationData);
            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), session_index)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetGameAuthenticationData, 1, 2),
                Ok(()),
            );
            assert_eq!(result.validate_buffer_id(2, 0), Ok(()));
            assert_eq!(result.pop_result(), Err(FrdErrorCode::MissingData.into()));

            let game_auth_data: Vec<GameAuthenticationData> =
                unsafe { result.pop_static_buffer().unwrap().to_vec() };
            assert_eq!(game_auth_data, vec![Default::default()])
        }
    }

    mod request_service_locator {
        use super::*;
        use ctr::http::{HttpContext, RequestMethod};
        use ctr::sysmodule::server::ServiceContext;
        use safe_transmute::transmute_to_bytes_mut;

        #[test]
        fn should_send_a_service_locate_request_made_from_the_given_parameters() {
            let locate_response = "retry=MA**&returncd=MDA3&servicetoken=AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8gISIjJCUmJygpKissLS4vMDE*&statusdata=WQ**&svchost=bi9h&datetime=MjAyMTAxMDIwMzA0MDU*";

            let downloaded_locate_response = locate_response.clone();
            HttpContext::download_data_into_buffer.mock_safe(move |_, buffer| {
                let response_bytes = downloaded_locate_response.as_bytes();
                buffer[..response_bytes.len()].clone_from_slice(response_bytes);
                MockResult::Return(Ok(()))
            });

            create_game_service_locate_request.mock_safe(
                |_,
                 requesting_process_id,
                 requesting_game_id,
                 sdk_version_low,
                 sdk_version_high,
                 key_hash,
                 svc| {
                    assert_eq!(requesting_process_id, 0);
                    assert_eq!(requesting_game_id, 1234);
                    assert_eq!(sdk_version_low, 3);
                    assert_eq!(sdk_version_high, 4);
                    assert_eq!(key_hash, "keyhash-test");
                    assert_eq!(svc, "svc-test");
                    MockResult::Return(HttpContext::new("", RequestMethod::Post))
                },
            );

            let mut key_hash: [u32; 3] = [0; 3];
            let key_hash_buffer = transmute_to_bytes_mut(&mut key_hash);
            key_hash_buffer.copy_from_slice("keyhash-test".as_bytes());

            let mut svc: [u32; 2] = [0; 2];
            let svc_buffer = transmute_to_bytes_mut(&mut svc);
            svc_buffer.copy_from_slice("svc-test".as_bytes());

            let session_index = 0usize;
            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.session_contexts[session_index].last_service_locator_response = None;

            let mut command = ThreadCommandBuilder::new(FrdUCommand::RequestServiceLocator);
            command.push(1234u32);
            command.push_struct(&key_hash);
            command.push_struct(&svc);
            command.push(3u32);
            command.push(4u32);
            command.push_curent_process_id();
            command.push_raw_handle(5678u32);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), session_index)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::RequestServiceLocator, 1, 0),
                Ok(()),
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(
                context.session_contexts[session_index]
                    .last_service_locator_response
                    .unwrap(),
                ServiceLocateData::from_fetched_response(locate_response, 200).unwrap()
            );
        }

        #[test]
        fn should_ignore_key_hash_and_svc_bytes_after_null_terminator() {
            create_game_service_locate_request.mock_safe(
                |_,
                 _requesting_process_id,
                 _requesting_game_id,
                 _sdk_version_low,
                 _sdk_version_high,
                 key_hash,
                 svc| {
                    assert_eq!(key_hash, "");
                    assert_eq!(svc, "");
                    MockResult::Return(HttpContext::new("", RequestMethod::Post))
                },
            );

            let mut key_hash: [u32; 3] = [0; 3];
            let key_hash_buffer = transmute_to_bytes_mut(&mut key_hash);
            key_hash_buffer[1..9].copy_from_slice("key_hash".as_bytes());

            let mut svc: [u32; 2] = [0; 2];
            let svc_buffer = transmute_to_bytes_mut(&mut svc);
            svc_buffer[1..4].copy_from_slice("svc".as_bytes());

            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();

            let mut command = ThreadCommandBuilder::new(FrdUCommand::RequestServiceLocator);
            command.push(1234u32);
            command.push_struct(&key_hash);
            command.push_struct(&svc);
            command.push(3u32);
            command.push(4u32);
            command.push_curent_process_id();
            command.push_raw_handle(5678u32);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::RequestServiceLocator, 1, 0),
                Ok(()),
            );
            assert_eq!(result.pop_result(), Ok(()));
        }

        #[test]
        fn should_set_the_server_time_interval() {
            calculate_time_difference_from_now.mock_safe(|_| MockResult::Return(0xabcd));
            create_game_service_locate_request.mock_safe(
                |_,
                 _requesting_process_id,
                 _requesting_game_id,
                 _sdk_version_low,
                 _sdk_version_high,
                 _key_hash,
                 _svc| {
                    MockResult::Return(HttpContext::new("", RequestMethod::Post))
                },
            );

            let mut key_hash: [u32; 3] = [0; 3];
            let key_hash_buffer = transmute_to_bytes_mut(&mut key_hash);
            key_hash_buffer[1..9].copy_from_slice("key_hash".as_bytes());

            let mut svc: [u32; 2] = [0; 2];
            let svc_buffer = transmute_to_bytes_mut(&mut svc);
            svc_buffer[1..4].copy_from_slice("svc".as_bytes());

            let session_index = 0;
            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.session_contexts[session_index].server_time_interval = 0;

            let mut command = ThreadCommandBuilder::new(FrdUCommand::RequestServiceLocator);
            command.push(1234u32);
            command.push_struct(&key_hash);
            command.push_struct(&svc);
            command.push(3u32);
            command.push(4u32);
            command.push_curent_process_id();
            command.push_raw_handle(5678u32);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), session_index)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::RequestServiceLocator, 1, 0),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(
                context.session_contexts[session_index].server_time_interval,
                0xabcd,
            );
        }
    }

    mod get_service_locator_data {
        use super::*;
        use crate::frd::online_play::locate::ServiceLocateData;
        use alloc::vec;
        use ctr::sysmodule::server::ServiceContext;

        #[test]
        fn should_return_success_if_locator_data_exists() {
            let service_locator_response = ServiceLocateData {
                status_data: [1, 2, 3, 4, 5, 6, 7, 8],
                ..Default::default()
            };

            let session_index = 0usize;
            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.session_contexts[session_index].last_service_locator_response =
                Some(service_locator_response);

            let command = ThreadCommandBuilder::new(FrdUCommand::GetServiceLocatorData);
            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), session_index)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetServiceLocatorData, 1, 2),
                Ok(())
            );
            assert_eq!(result.validate_buffer_id(2, 0), Ok(()));
            assert_eq!(result.pop_result(), Ok(()));

            let locate_data: Vec<ServiceLocateData> =
                unsafe { result.pop_static_buffer().unwrap().to_vec() };
            assert_eq!(locate_data, vec![service_locator_response])
        }

        #[test]
        fn should_return_error_if_auth_data_does_not_exist() {
            let session_index = 0usize;
            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.session_contexts[session_index].last_service_locator_response = None;

            let command = ThreadCommandBuilder::new(FrdUCommand::GetServiceLocatorData);
            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), session_index)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetServiceLocatorData, 1, 2),
                Ok(())
            );
            assert_eq!(result.validate_buffer_id(2, 0), Ok(()));
            assert_eq!(result.pop_result(), Err(FrdErrorCode::MissingData.into()));

            let locate_data: Vec<ServiceLocateData> =
                unsafe { result.pop_static_buffer().unwrap().to_vec() };
            assert_eq!(locate_data, vec![Default::default()])
        }
    }

    mod detect_nat_properties {
        use super::*;

        #[test]
        fn should_signal_the_given_event() {
            let mock_handle: u32 = 0x1234;

            svc::signal_event.mock_safe(move |handle| {
                // This is safe since it's a test, the handle isn't real
                // and it won't be used for anything.
                let raw_handle = unsafe { handle.get_raw() };
                assert_eq!(raw_handle, mock_handle);
                MockResult::Return(Ok(()))
            });

            let mut context = FriendServiceContext::new().unwrap();
            let mut command = ThreadCommandBuilder::new(FrdUCommand::DetectNatProperties);
            command.push_raw_handle(mock_handle);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::DetectNatProperties, 1, 0),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
        }
    }

    mod get_nat_properties {
        use super::*;
        use ctr::frd::NatProperties;

        #[test]
        fn should_return_nat_properties() {
            let mut context = FriendServiceContext::new().unwrap();
            context.nat_properties = NatProperties::new(1, 2, 3);

            let command = ThreadCommandBuilder::new(FrdUCommand::GetNatProperties);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetNatProperties, 3, 0),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(result.pop(), context.nat_properties.get_unk1() as u32);
            assert_eq!(result.pop(), context.nat_properties.get_unk2() as u32);
        }
    }

    mod get_server_time_interval {
        use super::*;
        use ctr::sysmodule::server::ServiceContext;

        #[test]
        fn should_return_the_saved_server_time_interval() {
            let session_index = 0;
            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.session_contexts[session_index].server_time_interval = 0xaaaabbbbccccdddd;

            let command = ThreadCommandBuilder::new(FrdUCommand::GetServerTimeInterval);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), session_index)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetServerTimeInterval, 3, 0),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(result.pop_u64(), 0xaaaabbbbccccdddd);
        }
    }

    mod set_client_sdk_version {
        use super::*;
        use ctr::sysmodule::server::ServiceContext;

        #[test]
        fn should_set_the_client_sdk_version_and_process_id() {
            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();

            let session_index = 0;
            let session_context = &mut context.session_contexts[session_index];
            session_context.client_sdk_version = 1;
            session_context.process_id = 1;

            let mut command = ThreadCommandBuilder::new(FrdUCommand::SetClientSdkVersion);
            command.push(0xaabbu32);
            command.push_curent_process_id();

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), session_index)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::SetClientSdkVersion, 1, 0),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));

            let session_context = &context.session_contexts[session_index];
            assert_eq!(session_context.client_sdk_version, 0xaabb);
            assert_eq!(session_context.process_id, 0);
        }

        #[test]
        fn should_error_if_a_non_process_id_translate_param_was_provided() {
            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();

            let mut fake_out: [u8; 1] = [0];
            let mut command = ThreadCommandBuilder::new(FrdUCommand::SetClientSdkVersion);
            command.push(0xaabbu32);
            command.push_write_buffer(&mut fake_out);

            let result = handle_frdu_request(&mut context, command.build().into(), 0).unwrap_err();

            assert_eq!(result, GenericResultCode::InvalidValue.into_result_code());
        }

        #[test]
        fn should_error_if_an_invalid_normal_param_count_is_given() {
            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();

            let mut command = ThreadCommandBuilder::new(FrdUCommand::SetClientSdkVersion);
            command.push(0xaabbu32);
            command.push(0xaabbu32);
            command.push_curent_process_id();

            let result = handle_frdu_request(&mut context, command.build().into(), 0).unwrap_err();

            assert_eq!(result, GenericResultCode::InvalidCommand.into_result_code());
        }

        #[test]
        fn should_error_if_an_translate_normal_param_count_is_given() {
            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();

            let mut command = ThreadCommandBuilder::new(FrdUCommand::SetClientSdkVersion);
            command.push(0xaabbu32);
            command.push_curent_process_id();
            command.push_curent_process_id();

            let result = handle_frdu_request(&mut context, command.build().into(), 0).unwrap_err();

            assert_eq!(result, GenericResultCode::InvalidCommand.into_result_code());
        }
    }

    mod get_server_types {
        use super::*;
        use crate::frd::save::account::NascEnvironment;

        #[test]
        fn should_return_the_server_types() {
            let mut context = FriendServiceContext::new().unwrap();
            context.account_config.nasc_environment = NascEnvironment::Prod;
            context.account_config.server_type_1 = 0xa;
            context.account_config.server_type_2 = 0xb;

            let command = ThreadCommandBuilder::new(FrdUCommand::GetServerTypes);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetServerTypes, 4, 0),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(result.pop(), context.account_config.nasc_environment as u32);
            assert_eq!(result.pop(), context.account_config.server_type_1 as u32);
            assert_eq!(result.pop(), context.account_config.server_type_2 as u32);
        }
    }

    mod get_friend_comment {
        use super::*;
        use crate::frd::save::friend_list::FriendEntry;
        use alloc::vec;
        use ctr::sysmodule::server::ServiceContext;

        #[test]
        fn should_return_comments_for_each_friend() {
            let friend_key_1 = FriendKey {
                principal_id: 1,
                ..Default::default()
            };
            let friend_key_2 = FriendKey {
                principal_id: 2,
                ..Default::default()
            };

            let friend_1 = FriendEntry {
                friend_key: friend_key_1,
                friend_relationship: 1,
                comment: "Friend 1".into(),
                ..Default::default()
            };
            let friend_2 = FriendEntry {
                friend_key: friend_key_2,
                friend_relationship: 2,
                comment: "Friend 2".into(),
                ..Default::default()
            };

            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.friend_list.clear();
            context.friend_list.push(friend_1);
            context.friend_list.push(friend_2);

            let friend_key_list = [friend_key_1, friend_key_2];
            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendComment);
            command.push(friend_key_list.len() as u32);
            command.push(0u32);
            command.push_static_buffer(&friend_key_list, 0);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetFriendComment, 1, 2),
                Ok(())
            );
            assert_eq!(result.pop(), u32::from(ResultCode::success()));

            let comments: Vec<FriendComment> =
                unsafe { result.pop_static_buffer().unwrap().to_vec() };
            assert_eq!(comments, vec!["Friend 1".into(), "Friend 2".into()]);
        }

        #[test]
        fn should_fail_if_the_header_has_an_incorrect_normal_param_count() {
            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.friend_list.clear();

            let friend_key_list: [FriendKey; 0] = [];

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendComment);
            command.push(0u32);
            command.push_static_buffer(&friend_key_list, 0);

            let result = handle_frdu_request(&mut context, command.build().into(), 0).unwrap_err();

            assert_eq!(result, GenericResultCode::InvalidCommand.into_result_code())
        }

        #[test]
        fn should_fail_if_the_header_has_an_incorrect_translate_param_count() {
            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.friend_list.clear();

            let friend_key_list: [FriendKey; 0] = [];

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendComment);
            command.push(friend_key_list.len() as u32);
            command.push_curent_process_id();
            command.push_curent_process_id();

            let result = handle_frdu_request(&mut context, command.build().into(), 0).unwrap_err();

            assert_eq!(result, GenericResultCode::InvalidCommand.into_result_code())
        }

        #[test]
        fn should_fail_if_the_friend_key_buffer_id_is_not_0() {
            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.friend_list.clear();

            let friend_key_list: [FriendKey; 0] = [];

            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendComment);
            command.push(friend_key_list.len() as u32);
            command.push(0u32);
            command.push_static_buffer(&friend_key_list, 1);

            let result = handle_frdu_request(&mut context, command.build().into(), 0).unwrap_err();

            assert_eq!(result, GenericResultCode::InvalidCommand.into_result_code())
        }

        #[test]
        fn should_limit_the_number_of_results_to_the_number_of_friend_keys() {
            let friend_key_1 = FriendKey {
                principal_id: 1,
                ..Default::default()
            };
            let friend_key_2 = FriendKey {
                principal_id: 2,
                ..Default::default()
            };

            let friend_1 = FriendEntry {
                friend_key: friend_key_1,
                friend_relationship: 1,
                ..Default::default()
            };
            let friend_2 = FriendEntry {
                friend_key: friend_key_2,
                friend_relationship: 2,
                ..Default::default()
            };

            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.friend_list.clear();
            context.friend_list.push(friend_1);
            context.friend_list.push(friend_2);

            let friend_key_list = [friend_key_2];
            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendComment);
            command.push(friend_key_list.len() as u32);
            command.push(0u32);
            command.push_static_buffer(&friend_key_list, 0);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetFriendComment, 1, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            let comments: Vec<FriendComment> =
                unsafe { result.pop_static_buffer().unwrap().to_vec() };
            assert_eq!(comments.len(), 1);
        }

        #[test]
        fn should_return_an_empty_comment_for_any_friends_that_are_not_found() {
            let friend_key_1 = FriendKey {
                principal_id: 1,
                ..Default::default()
            };
            let friend_key_2 = FriendKey {
                principal_id: 2,
                ..Default::default()
            };

            let friend_1 = FriendEntry {
                friend_key: friend_key_1,
                friend_relationship: 1,
                comment: "Friend 1".into(),
                ..Default::default()
            };

            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.friend_list.clear();
            context.friend_list.push(friend_1);

            let friend_key_list = [friend_key_1, friend_key_2];
            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendComment);
            command.push(friend_key_list.len() as u32);
            command.push(0u32);
            command.push_static_buffer(&friend_key_list, 0);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetFriendComment, 1, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            let comments: Vec<FriendComment> =
                unsafe { result.pop_static_buffer().unwrap().to_vec() };
            assert_eq!(comments, vec!["Friend 1".into(), "".into()]);
        }

        #[test]
        fn should_limit_to_100_friends_max() {
            let friend_key_1 = FriendKey {
                principal_id: 1,
                ..Default::default()
            };
            let friend_1 = FriendEntry {
                friend_key: friend_key_1,
                friend_relationship: 1,
                ..Default::default()
            };

            let mut context = FriendServiceContext::new().unwrap();
            context.accept_session();
            context.friend_list.clear();
            context.friend_list.push(friend_1);

            let friend_key_list: [FriendKey; 200] = [friend_key_1; 200];
            let mut command = ThreadCommandBuilder::new(FrdUCommand::GetFriendComment);
            command.push(friend_key_list.len() as u32);
            command.push(0u32);
            command.push_static_buffer(&friend_key_list, 0);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetFriendComment, 1, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            let comments: Vec<FriendComment> =
                unsafe { result.pop_static_buffer().unwrap().to_vec() };
            assert_eq!(comments.len(), 100);
        }
    }

    mod get_extended_nat_properties {
        use super::*;
        use ctr::frd::NatProperties;

        #[test]
        fn should_return_nat_properties() {
            let mut context = FriendServiceContext::new().unwrap();
            context.nat_properties = NatProperties::new(1, 2, 3);

            let command = ThreadCommandBuilder::new(FrdUCommand::GetExtendedNatProperties);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::GetExtendedNatProperties, 4, 0),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(result.pop(), context.nat_properties.get_unk1() as u32);
            assert_eq!(result.pop(), context.nat_properties.get_unk2() as u32);
            assert_eq!(result.pop(), context.nat_properties.get_unk3());
        }
    }

    mod unknown_cmd {
        use super::*;

        #[test]
        fn handle_unknown_cmd() {
            let mut context = FriendServiceContext::new().unwrap();
            let command = ThreadCommandBuilder::new(FrdUCommand::InvalidCommand);

            let mut result: ThreadCommandParser =
                handle_frdu_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdUCommand::InvalidCommand, 1, 0),
                Ok(())
            );
            assert_eq!(
                result.pop_result().unwrap_err(),
                FrdErrorCode::InvalidCommand.into_result_code()
            );
        }
    }
}
