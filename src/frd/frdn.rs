use crate::frd::{
    context::FriendServiceContext,
    result::FrdErrorCode,
    wifi::{connect_to_wifi, get_wifi_state, set_wifi_connection_status, WiFiConnectionStatus},
};
use core::convert::From;
use ctr::{
    ac::AcController,
    ipc::{ThreadCommandBuilder, ThreadCommandParser},
    result::ResultCode,
    svc,
    sysmodule::server::RequestHandlerResult,
};
use num_enum::{FromPrimitive, IntoPrimitive};

#[derive(IntoPrimitive, FromPrimitive)]
#[repr(u16)]
enum FrdNCommand {
    #[num_enum(default)]
    InvalidCommand = 0,
    GetWiFiEvent = 1,
    ConnectToWiFi = 2,
    DisconnectFromWiFi = 3,
    GetWiFiState = 4,
}

pub fn handle_frdn_request(
    context: &mut FriendServiceContext,
    mut command_parser: ThreadCommandParser,
    _session_index: usize,
) -> RequestHandlerResult {
    let command_id = command_parser.get_command_id();

    match command_id.into() {
        FrdNCommand::GetWiFiEvent => {
            // This is safe since we're sending it to another process, not copying it
            let raw_event_handle = unsafe { context.ndm_wifi_event_handle.get_raw() };

            let mut command = ThreadCommandBuilder::new(FrdNCommand::GetWiFiEvent);
            command.push(ResultCode::success());
            command.push_raw_handle(raw_event_handle);

            Ok(command.build())
        }
        FrdNCommand::ConnectToWiFi => {
            connect_to_wifi(context)?;

            let mut command = ThreadCommandBuilder::new(FrdNCommand::ConnectToWiFi);
            command.push(0u32);
            Ok(command.build())
        }
        FrdNCommand::DisconnectFromWiFi => {
            let connection_status = context.wifi_connection_status;
            let original_ndm_wifi_state = context.ndm_wifi_state;
            let next_state = command_parser.pop() as u8;
            context.ndm_wifi_state = next_state ^ 1;

            if connection_status == WiFiConnectionStatus::Connected {
                set_wifi_connection_status(context, WiFiConnectionStatus::Disconnecting)?;
                AcController::disconnect()?;
                set_wifi_connection_status(context, WiFiConnectionStatus::Idle)?;
            } else if original_ndm_wifi_state == 2 {
                svc::signal_event(&context.ndm_wifi_event_handle)?;
            }

            let mut command = ThreadCommandBuilder::new(FrdNCommand::DisconnectFromWiFi);
            command.push(ResultCode::success());
            Ok(command.build())
        }
        FrdNCommand::GetWiFiState => {
            let result = get_wifi_state(context.ndm_wifi_state, context.wifi_connection_status);

            let mut command = ThreadCommandBuilder::new(FrdNCommand::GetWiFiState);
            command.push(ResultCode::success());
            command.push(result);
            Ok(command.build())
        }
        FrdNCommand::InvalidCommand => {
            let mut command = ThreadCommandBuilder::new(FrdNCommand::InvalidCommand);
            command.push(FrdErrorCode::InvalidCommand);
            Ok(command.build())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod get_wifi_event {
        use super::*;

        #[test]
        fn get_wifi_event_should_return_the_wifi_event_handle() {
            let mut context = FriendServiceContext::new().unwrap();
            let command = ThreadCommandBuilder::new(FrdNCommand::GetWiFiEvent);

            let mut result: ThreadCommandParser =
                handle_frdn_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdNCommand::GetWiFiEvent, 1, 2),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(result.pop_handle().unwrap(), context.ndm_wifi_event_handle);
        }
    }

    mod connect_to_wifi {
        use super::*;
        use mocktopus::mocking::{MockResult, Mockable};

        #[test]
        fn should_return_success_if_connecting_is_successful() {
            connect_to_wifi.mock_safe(|_| MockResult::Return(Ok(())));

            let mut context = FriendServiceContext::new().unwrap();
            let command = ThreadCommandBuilder::new(FrdNCommand::ConnectToWiFi);

            let mut result: ThreadCommandParser =
                handle_frdn_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdNCommand::ConnectToWiFi, 1, 0),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
        }

        #[test]
        fn should_return_an_error_if_connecting_is_not_successful() {
            connect_to_wifi.mock_safe(|_| MockResult::Return(Err(ResultCode::from(-1))));

            let mut context = FriendServiceContext::new().unwrap();
            let command = ThreadCommandBuilder::new(FrdNCommand::ConnectToWiFi);

            let result = handle_frdn_request(&mut context, command.build().into(), 0).unwrap_err();

            assert_eq!(result, ResultCode::from(-1));
        }
    }

    mod disconnect_from_wifi {
        use mocktopus::mocking::Mockable;

        use super::*;

        #[test]
        fn should_set_the_ndm_state_to_the_pushed_parameter_with_bit_0_flipped() {
            let mut context = FriendServiceContext::new().unwrap();
            context.wifi_connection_status = WiFiConnectionStatus::Connected;
            context.ndm_wifi_state = 1;
            let mut command = ThreadCommandBuilder::new(FrdNCommand::DisconnectFromWiFi);
            command.push(1u32);

            let mut result: ThreadCommandParser =
                handle_frdn_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdNCommand::DisconnectFromWiFi, 1, 0),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(context.ndm_wifi_state, 0);
        }

        #[test]
        fn should_set_the_wifi_connection_status_to_idle_if_the_wifi_connection_status_is_connected(
        ) {
            let mut context = FriendServiceContext::new().unwrap();
            context.wifi_connection_status = WiFiConnectionStatus::Connected;
            context.ndm_wifi_state = 0;
            let mut command = ThreadCommandBuilder::new(FrdNCommand::DisconnectFromWiFi);
            command.push(0u32);

            let mut result: ThreadCommandParser =
                handle_frdn_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdNCommand::DisconnectFromWiFi, 1, 0),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(context.wifi_connection_status, WiFiConnectionStatus::Idle);
        }

        #[test]
        fn should_not_call_disconnect_if_the_wifi_connection_status_is_not_connected() {
            AcController::disconnect
                .mock_safe(|| panic!("WiFi should not have been disconnected!"));

            let mut context = FriendServiceContext::new().unwrap();
            context.wifi_connection_status = WiFiConnectionStatus::Idle;
            context.ndm_wifi_state = 0;
            let mut command = ThreadCommandBuilder::new(FrdNCommand::DisconnectFromWiFi);
            command.push(0u32);

            let mut result: ThreadCommandParser =
                handle_frdn_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdNCommand::DisconnectFromWiFi, 1, 0),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
        }

        #[test]
        fn should_not_signal_the_event_if_the_ndm_state_is_0_and_the_wifi_connection_status_is_not_connected(
        ) {
            svc::signal_event.mock_safe(|_| panic!("Event should not have been signalled!"));

            let mut context = FriendServiceContext::new().unwrap();
            context.wifi_connection_status = WiFiConnectionStatus::Idle;
            context.ndm_wifi_state = 0;
            let mut command = ThreadCommandBuilder::new(FrdNCommand::DisconnectFromWiFi);
            command.push(0u32);

            let mut result: ThreadCommandParser =
                handle_frdn_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdNCommand::DisconnectFromWiFi, 1, 0),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
        }

        #[test]
        fn should_not_signal_the_event_if_the_ndm_state_is_1_and_the_wifi_connection_status_is_not_connected(
        ) {
            svc::signal_event.mock_safe(|_| panic!("Event should not have been signalled!"));

            let mut context = FriendServiceContext::new().unwrap();
            context.wifi_connection_status = WiFiConnectionStatus::Idle;
            context.ndm_wifi_state = 1;
            let mut command = ThreadCommandBuilder::new(FrdNCommand::DisconnectFromWiFi);
            command.push(0u32);

            let mut result: ThreadCommandParser =
                handle_frdn_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdNCommand::DisconnectFromWiFi, 1, 0),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
        }
    }

    mod get_wifi_state {
        use super::*;

        #[test]
        fn should_return_2_when_the_ndm_state_is_0_and_the_wifi_connection_status_is_connected() {
            let mut context = FriendServiceContext::new().unwrap();
            context.ndm_wifi_state = 0;
            context.wifi_connection_status = WiFiConnectionStatus::Connected;
            let command = ThreadCommandBuilder::new(FrdNCommand::GetWiFiState);

            let mut result: ThreadCommandParser =
                handle_frdn_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdNCommand::GetWiFiState, 2, 0),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(result.pop(), 2);
        }

        #[test]
        fn should_return_2_when_the_ndm_state_is_0_and_the_wifi_connection_status_is_connecting() {
            let mut context = FriendServiceContext::new().unwrap();
            context.ndm_wifi_state = 0;
            context.wifi_connection_status = WiFiConnectionStatus::Connecting;
            let command = ThreadCommandBuilder::new(FrdNCommand::GetWiFiState);

            let mut result: ThreadCommandParser =
                handle_frdn_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdNCommand::GetWiFiState, 2, 0),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(result.pop(), 2);
        }

        #[test]
        fn should_return_2_when_the_ndm_state_is_0_and_the_wifi_connection_status_is_disconnecting()
        {
            let mut context = FriendServiceContext::new().unwrap();
            context.ndm_wifi_state = 0;
            context.wifi_connection_status = WiFiConnectionStatus::Disconnecting;
            let command = ThreadCommandBuilder::new(FrdNCommand::GetWiFiState);

            let mut result: ThreadCommandParser =
                handle_frdn_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdNCommand::GetWiFiState, 2, 0),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(result.pop(), 2);
        }

        #[test]
        fn should_return_3_when_the_ndm_state_is_0_and_the_wifi_connection_status_is_idle() {
            let mut context = FriendServiceContext::new().unwrap();
            context.ndm_wifi_state = 0;
            context.wifi_connection_status = WiFiConnectionStatus::Idle;
            let command = ThreadCommandBuilder::new(FrdNCommand::GetWiFiState);

            let mut result: ThreadCommandParser =
                handle_frdn_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdNCommand::GetWiFiState, 2, 0),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(result.pop(), 3);
        }

        #[test]
        fn should_return_2_when_the_ndm_state_is_1_and_the_wifi_connection_status_is_connected() {
            let mut context = FriendServiceContext::new().unwrap();
            context.ndm_wifi_state = 1;
            context.wifi_connection_status = WiFiConnectionStatus::Connected;
            let command = ThreadCommandBuilder::new(FrdNCommand::GetWiFiState);

            let mut result: ThreadCommandParser =
                handle_frdn_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdNCommand::GetWiFiState, 2, 0),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(result.pop(), 2);
        }

        #[test]
        fn should_return_2_when_the_ndm_state_is_1_and_the_wifi_connection_status_is_connecting() {
            let mut context = FriendServiceContext::new().unwrap();
            context.ndm_wifi_state = 1;
            context.wifi_connection_status = WiFiConnectionStatus::Connecting;
            let command = ThreadCommandBuilder::new(FrdNCommand::GetWiFiState);

            let mut result: ThreadCommandParser =
                handle_frdn_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdNCommand::GetWiFiState, 2, 0),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(result.pop(), 2);
        }

        #[test]
        fn should_return_2_when_the_ndm_state_is_1_and_the_wifi_connection_status_is_disconnecting()
        {
            let mut context = FriendServiceContext::new().unwrap();
            context.ndm_wifi_state = 1;
            context.wifi_connection_status = WiFiConnectionStatus::Disconnecting;
            let command = ThreadCommandBuilder::new(FrdNCommand::GetWiFiState);

            let mut result: ThreadCommandParser =
                handle_frdn_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdNCommand::GetWiFiState, 2, 0),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(result.pop(), 2);
        }

        #[test]
        fn should_return_3_when_the_ndm_state_is_1_and_the_wifi_connection_status_is_idle() {
            let mut context = FriendServiceContext::new().unwrap();
            context.ndm_wifi_state = 1;
            context.wifi_connection_status = WiFiConnectionStatus::Idle;
            let command = ThreadCommandBuilder::new(FrdNCommand::GetWiFiState);

            let mut result: ThreadCommandParser =
                handle_frdn_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdNCommand::GetWiFiState, 2, 0),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(result.pop(), 3);
        }

        #[test]
        fn should_return_0_when_the_ndm_state_is_2_and_the_wifi_connection_status_is_connected() {
            let mut context = FriendServiceContext::new().unwrap();
            context.ndm_wifi_state = 2;
            context.wifi_connection_status = WiFiConnectionStatus::Connected;
            let command = ThreadCommandBuilder::new(FrdNCommand::GetWiFiState);

            let mut result: ThreadCommandParser =
                handle_frdn_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdNCommand::GetWiFiState, 2, 0),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(result.pop(), 0);
        }

        #[test]
        fn should_return_0_when_the_ndm_state_is_2_and_the_wifi_connection_status_is_connecting() {
            let mut context = FriendServiceContext::new().unwrap();
            context.ndm_wifi_state = 2;
            context.wifi_connection_status = WiFiConnectionStatus::Connecting;
            let command = ThreadCommandBuilder::new(FrdNCommand::GetWiFiState);

            let mut result: ThreadCommandParser =
                handle_frdn_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdNCommand::GetWiFiState, 2, 0),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(result.pop(), 0);
        }

        #[test]
        fn should_return_0_when_the_ndm_state_is_2_and_the_wifi_connection_status_is_disconnecting()
        {
            let mut context = FriendServiceContext::new().unwrap();
            context.ndm_wifi_state = 2;
            context.wifi_connection_status = WiFiConnectionStatus::Disconnecting;
            let command = ThreadCommandBuilder::new(FrdNCommand::GetWiFiState);

            let mut result: ThreadCommandParser =
                handle_frdn_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdNCommand::GetWiFiState, 2, 0),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(result.pop(), 0);
        }

        #[test]
        fn should_return_1_when_the_ndm_state_is_2_and_the_wifi_connection_status_is_idle() {
            let mut context = FriendServiceContext::new().unwrap();
            context.ndm_wifi_state = 2;
            context.wifi_connection_status = WiFiConnectionStatus::Idle;
            let command = ThreadCommandBuilder::new(FrdNCommand::GetWiFiState);

            let mut result: ThreadCommandParser =
                handle_frdn_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdNCommand::GetWiFiState, 2, 0),
                Ok(())
            );
            assert_eq!(result.pop_result(), Ok(()));
            assert_eq!(result.pop(), 1);
        }
    }

    mod unknown_cmd {
        use super::*;

        #[test]
        fn handle_unknown_cmd() {
            let mut context = FriendServiceContext::new().unwrap();
            let command = ThreadCommandBuilder::new(FrdNCommand::InvalidCommand);

            let mut result: ThreadCommandParser =
                handle_frdn_request(&mut context, command.build().into(), 0)
                    .unwrap()
                    .into();

            assert_eq!(
                result.validate_header(FrdNCommand::InvalidCommand, 1, 0),
                Ok(())
            );
            assert_eq!(
                result.pop_result().unwrap_err(),
                FrdErrorCode::InvalidCommand.into_result_code()
            );
        }
    }
}
