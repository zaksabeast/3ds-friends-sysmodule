use super::WiFiConnectionStatus;
use crate::frd::context::FriendServiceContext;
use ctr::{ac::AcController, result::CtrResult, svc};

pub fn get_wifi_state(ndm_wifi_state: u8, wifi_connection_status: WiFiConnectionStatus) -> u32 {
    match (ndm_wifi_state, wifi_connection_status) {
        (0, WiFiConnectionStatus::Connecting) => 2,
        (0, WiFiConnectionStatus::Connected) => 2,
        (0, WiFiConnectionStatus::Disconnecting) => 2,
        (1, WiFiConnectionStatus::Connecting) => 2,
        (1, WiFiConnectionStatus::Connected) => 2,
        (1, WiFiConnectionStatus::Disconnecting) => 2,
        (2, WiFiConnectionStatus::Idle) => 1,
        (2, _) => 0,
        (_, _) => 3,
    }
}

pub fn set_wifi_connection_status(
    context: &mut FriendServiceContext,
    next_wifi_connection_status: WiFiConnectionStatus,
) -> CtrResult<()> {
    if context.wifi_connection_status != next_wifi_connection_status {
        let old_state = get_wifi_state(context.ndm_wifi_state, context.wifi_connection_status);
        context.wifi_connection_status = next_wifi_connection_status;
        let new_state = get_wifi_state(context.ndm_wifi_state, context.wifi_connection_status);

        if old_state != new_state {
            svc::signal_event(&context.ndm_wifi_event_handle)?;
        }
    }

    Ok(())
}

#[cfg_attr(test, mocktopus::macros::mockable)]
pub fn connect_to_wifi(context: &mut FriendServiceContext) -> CtrResult<()> {
    let original_ndm_wifi_state = context.ndm_wifi_state;
    context.ndm_wifi_state = 2;

    if context.wifi_connection_status == WiFiConnectionStatus::Idle {
        set_wifi_connection_status(context, WiFiConnectionStatus::Connecting)?;

        return match AcController::quick_connect() {
            Ok(_) => {
                set_wifi_connection_status(context, WiFiConnectionStatus::Connected)?;
                Ok(())
            }
            Err(result_code) => {
                set_wifi_connection_status(context, WiFiConnectionStatus::Idle)?;
                Err(result_code)
            }
        };
    }

    if original_ndm_wifi_state != 2 {
        svc::signal_event(&context.ndm_wifi_event_handle)?;
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    mod get_wifi_state {
        use super::*;

        #[test]
        fn should_return_2_when_the_ndm_state_is_0_and_the_wifi_connection_status_is_connected() {
            let result = get_wifi_state(0, WiFiConnectionStatus::Connected);
            assert_eq!(result, 2);
        }

        #[test]
        fn should_return_2_when_the_ndm_state_is_0_and_the_wifi_connection_status_is_connecting() {
            let result = get_wifi_state(0, WiFiConnectionStatus::Connecting);
            assert_eq!(result, 2);
        }

        #[test]
        fn should_return_2_when_the_ndm_state_is_0_and_the_wifi_connection_status_is_disconnecting()
        {
            let result = get_wifi_state(0, WiFiConnectionStatus::Disconnecting);
            assert_eq!(result, 2);
        }

        #[test]
        fn should_return_3_when_the_ndm_state_is_0_and_the_wifi_connection_status_is_idle() {
            let result = get_wifi_state(0, WiFiConnectionStatus::Idle);
            assert_eq!(result, 3);
        }

        #[test]
        fn should_return_2_when_the_ndm_state_is_1_and_the_wifi_connection_status_is_connected() {
            let result = get_wifi_state(1, WiFiConnectionStatus::Connected);
            assert_eq!(result, 2);
        }

        #[test]
        fn should_return_2_when_the_ndm_state_is_1_and_the_wifi_connection_status_is_connecting() {
            let result = get_wifi_state(1, WiFiConnectionStatus::Connecting);
            assert_eq!(result, 2);
        }

        #[test]
        fn should_return_2_when_the_ndm_state_is_1_and_the_wifi_connection_status_is_disconnecting()
        {
            let result = get_wifi_state(1, WiFiConnectionStatus::Disconnecting);
            assert_eq!(result, 2);
        }

        #[test]
        fn should_return_3_when_the_ndm_state_is_1_and_the_wifi_connection_status_is_idle() {
            let result = get_wifi_state(1, WiFiConnectionStatus::Idle);
            assert_eq!(result, 3);
        }

        #[test]
        fn should_return_0_when_the_ndm_state_is_2_and_the_wifi_connection_status_is_connected() {
            let result = get_wifi_state(2, WiFiConnectionStatus::Connected);
            assert_eq!(result, 0);
        }

        #[test]
        fn should_return_0_when_the_ndm_state_is_2_and_the_wifi_connection_status_is_connecting() {
            let result = get_wifi_state(2, WiFiConnectionStatus::Connecting);
            assert_eq!(result, 0);
        }

        #[test]
        fn should_return_0_when_the_ndm_state_is_2_and_the_wifi_connection_status_is_disconnecting()
        {
            let result = get_wifi_state(2, WiFiConnectionStatus::Disconnecting);
            assert_eq!(result, 0);
        }

        #[test]
        fn should_return_1_when_the_ndm_state_is_2_and_the_wifi_connection_status_is_idle() {
            let result = get_wifi_state(2, WiFiConnectionStatus::Idle);
            assert_eq!(result, 1);
        }
    }

    mod set_wifi_status {
        use mocktopus::mocking::{MockResult, Mockable};

        use super::*;

        #[test]
        fn should_signal_the_event_if_the_wifi_connection_status_and_next_state_is_different() {
            let raw_event_handle = 1234;

            svc::signal_event.mock_safe(move |handle| {
                // This is safe since it's just a test and the handle won't be duplicated.
                let raw_handle = unsafe { handle.get_raw() };
                assert_eq!(raw_handle, raw_event_handle);
                MockResult::Return(Ok(0))
            });

            let mut context = FriendServiceContext::new().unwrap();
            context.ndm_wifi_state = 1;
            context.wifi_connection_status = WiFiConnectionStatus::Idle;
            context.ndm_wifi_event_handle = raw_event_handle.into();

            set_wifi_connection_status(&mut context, WiFiConnectionStatus::Connecting).unwrap();
        }

        #[test]
        fn should_not_signal_the_event_if_the_wifi_connection_status_is_the_same_as_the_current_status(
        ) {
            let raw_event_handle = 1234;
            svc::signal_event.mock_safe(move |_| panic!("Event should not have been signalled!"));

            let mut context = FriendServiceContext::new().unwrap();
            context.ndm_wifi_state = 1;
            context.wifi_connection_status = WiFiConnectionStatus::Idle;
            context.ndm_wifi_event_handle = raw_event_handle.into();

            set_wifi_connection_status(&mut context, WiFiConnectionStatus::Idle).unwrap();
        }

        #[test]
        fn should_not_signal_the_event_if_the_state_is_the_same_as_the_current_state() {
            let raw_event_handle = 1234;
            svc::signal_event.mock_safe(move |_| panic!("Event should not have been signalled!"));

            let mut context = FriendServiceContext::new().unwrap();
            context.ndm_wifi_state = 1;
            context.wifi_connection_status = WiFiConnectionStatus::Connecting;
            context.ndm_wifi_event_handle = raw_event_handle.into();

            set_wifi_connection_status(&mut context, WiFiConnectionStatus::Connected).unwrap();
        }
    }

    mod connect_to_wifi {
        use super::*;
        use mocktopus::mocking::{MockResult, Mockable};

        #[test]
        fn should_signal_the_event_if_the_ndm_wifi_state_is_not_2_and_the_wifi_connection_status_is_not_idle(
        ) {
            let raw_event_handle = 1234;

            svc::signal_event.mock_safe(move |handle| {
                // This is safe since it's just a test and the handle won't be duplicated.
                let raw_handle = unsafe { handle.get_raw() };
                assert_eq!(raw_handle, raw_event_handle);
                MockResult::Return(Ok(0))
            });

            let mut context = FriendServiceContext::new().unwrap();
            context.ndm_wifi_state = 1;
            context.wifi_connection_status = WiFiConnectionStatus::Connecting;
            context.ndm_wifi_event_handle = raw_event_handle.into();

            connect_to_wifi(&mut context).unwrap();
        }

        #[test]
        fn should_set_the_wifi_status_to_connected_if_connecting_is_successful() {
            AcController::quick_connect.mock_safe(|| MockResult::Return(Ok(())));

            let mut context = FriendServiceContext::new().unwrap();
            context.ndm_wifi_state = 2;
            context.wifi_connection_status = WiFiConnectionStatus::Idle;
            context.ndm_wifi_event_handle = 1234u32.into();

            connect_to_wifi(&mut context).unwrap();

            assert_eq!(
                context.wifi_connection_status,
                WiFiConnectionStatus::Connected
            )
        }

        #[test]
        fn should_set_the_wifi_status_to_idle_if_connecting_is_not_successful() {
            AcController::quick_connect.mock_safe(|| MockResult::Return(Err(-1)));

            let mut context = FriendServiceContext::new().unwrap();
            context.ndm_wifi_state = 2;
            context.wifi_connection_status = WiFiConnectionStatus::Idle;
            context.ndm_wifi_event_handle = 1234u32.into();

            connect_to_wifi(&mut context).unwrap_err();

            assert_eq!(context.wifi_connection_status, WiFiConnectionStatus::Idle)
        }
    }
}
