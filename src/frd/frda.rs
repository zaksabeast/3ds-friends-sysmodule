use super::{context::FriendServiceContext, frdu::handle_frdu_request, result::FrdErrorCode};
use crate::log;
use alloc::format;
use core::convert::From;
use ctr::{
    frd::GameKey,
    ipc::{ThreadCommandBuilder, ThreadCommandParser},
    result::ResultCode,
    sysmodule::server::RequestHandlerResult,
};
use num_enum::{FromPrimitive, IntoPrimitive};

#[derive(IntoPrimitive, FromPrimitive)]
#[repr(u16)]
enum FrdACommand {
    #[num_enum(default)]
    InvalidCommand = 0,
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

pub fn handle_frda_request(
    context: &mut FriendServiceContext,
    mut command_parser: ThreadCommandParser,
    session_index: usize,
) -> RequestHandlerResult {
    let command_id = command_parser.get_command_id();

    if command_id < 0x400 {
        return handle_frdu_request(context, command_parser, session_index);
    }

    match command_id.into() {
        FrdACommand::CreateLocalAccount => {
            let _local_account_id = command_parser.pop();
            let _nasc_environment = command_parser.pop();
            let _server_type_field_1 = command_parser.pop();
            let _server_type_field_2 = command_parser.pop();

            let mut command = ThreadCommandBuilder::new(FrdACommand::CreateLocalAccount);
            command.push(ResultCode::success());
            Ok(command.build())
        }
        FrdACommand::HasUserData => {
            let mut command = ThreadCommandBuilder::new(FrdACommand::HasUserData);
            command.push(ResultCode::success());
            Ok(command.build())
        }
        FrdACommand::SetPresenseGameKey => {
            context.my_online_activity.playing_game = GameKey {
                title_id: command_parser.pop_u64(),
                version: command_parser.pop(),
                unk: command_parser.pop(),
            };

            log::debug(&format!(
                "SetPresenseGameKey {:08x}",
                context.my_online_activity.playing_game.title_id
            ));

            let mut command = ThreadCommandBuilder::new(FrdACommand::SetPresenseGameKey);
            command.push(ResultCode::success());
            Ok(command.build())
        }
        FrdACommand::SetMyData => {
            let mut command = ThreadCommandBuilder::new(FrdACommand::SetMyData);
            command.push(ResultCode::success());
            Ok(command.build())
        }
        _ => {
            let mut command = ThreadCommandBuilder::new(FrdACommand::InvalidCommand);
            command.push(FrdErrorCode::InvalidCommand);
            Ok(command.build())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod set_local_account_id_and_server_info {
        use super::*;

        #[test]
        fn returns_success() {
            let mut context = FriendServiceContext::new().unwrap();
            let command = ThreadCommandBuilder::new(FrdACommand::CreateLocalAccount);

            let mut result: ThreadCommandParser =
                handle_frda_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdACommand::CreateLocalAccount, 1, 0),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
        }
    }

    mod set_presense_game_key {
        use super::*;

        #[test]
        fn set_presence_game_key_should_set_the_provided_game_key() {
            let playing_game = GameKey {
                title_id: 0x1122334455667788,
                version: 0xAABBCCDD,
                ..Default::default()
            };

            let mut context = FriendServiceContext::new().unwrap();
            context.my_online_activity.playing_game = GameKey {
                title_id: 0,
                version: 0,
                ..Default::default()
            };

            let mut command = ThreadCommandBuilder::new(FrdACommand::SetPresenseGameKey);
            command.push_u64(playing_game.title_id);
            command.push(playing_game.version);
            command.push(playing_game.unk);

            let mut result: ThreadCommandParser =
                handle_frda_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdACommand::SetPresenseGameKey, 1, 0),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));

            assert_eq!(context.my_online_activity.playing_game, playing_game);
        }
    }

    mod set_my_data {
        use super::*;

        #[test]
        fn returns_success() {
            let mut context = FriendServiceContext::new().unwrap();
            let command = ThreadCommandBuilder::new(FrdACommand::SetMyData);

            let mut result: ThreadCommandParser =
                handle_frda_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(result.validate_header(FrdACommand::SetMyData, 1, 0), Ok(()));
            assert_eq!(result.pop_result(), Ok(()));
        }
    }

    mod unknown_cmd {
        use super::*;

        #[test]
        fn handle_unknown_cmd() {
            let mut context = FriendServiceContext::new().unwrap();
            let command = ThreadCommandBuilder::new(FrdACommand::InvalidCommand);

            let mut result: ThreadCommandParser =
                handle_frda_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdACommand::InvalidCommand, 1, 0),
                Ok(())
            );
            assert_eq!(
                result.pop_result().unwrap_err(),
                FrdErrorCode::InvalidCommand.into_result_code()
            );
        }
    }
}
