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
}
