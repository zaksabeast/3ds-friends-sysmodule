use crate::{frd::wifi, FriendSysmodule};
use alloc::vec;
use core::convert::From;
use ctr::{
    ac::AcController, ctr_method, ipc::Handles, res::CtrResult, svc, sysmodule::server::Service,
};
use num_enum::{FromPrimitive, IntoPrimitive};

#[derive(IntoPrimitive, FromPrimitive)]
#[repr(u16)]
pub enum FrdNCommand {
    #[num_enum(default)]
    InvalidCommand = 0,
    GetWiFiEvent = 1,
    ConnectToWiFi = 2,
    DisconnectFromWiFi = 3,
    GetWiFiState = 4,
}

impl Service for FrdNCommand {
    const ID: usize = 2;
    const NAME: &'static str = "frd:n";
    const MAX_SESSION_COUNT: i32 = 1;
}

#[ctr_method(cmd = "FrdNCommand::GetWiFiEvent", normal = 0x1, translate = 0x2)]
fn get_wifi_event(server: &mut FriendSysmodule, _session_index: usize) -> CtrResult<Handles> {
    let raw_handle = unsafe { server.context.ndm_wifi_event_handle.get_raw() };
    Ok(Handles::new(vec![raw_handle]))
}

#[ctr_method(cmd = "FrdNCommand::ConnectToWiFi", normal = 0x1, translate = 0x0)]
fn connect_to_wifi(server: &mut FriendSysmodule, _session_index: usize) -> CtrResult {
    wifi::connect_to_wifi(&mut server.context)?;
    Ok(())
}

#[ctr_method(cmd = "FrdNCommand::DisconnectFromWiFi", normal = 0x1, translate = 0x0)]
fn disconnect_from_wifi(
    server: &mut FriendSysmodule,
    _session_index: usize,
    next_state: u32,
) -> CtrResult {
    let next_state = next_state as u8;
    let connection_status = server.context.wifi_connection_status;
    let original_ndm_wifi_state = server.context.ndm_wifi_state;
    server.context.ndm_wifi_state = next_state ^ 1;

    if connection_status == wifi::WiFiConnectionStatus::Connected {
        wifi::set_wifi_connection_status(
            &mut server.context,
            wifi::WiFiConnectionStatus::Disconnecting,
        )?;
        AcController::disconnect()?;
        wifi::set_wifi_connection_status(&mut server.context, wifi::WiFiConnectionStatus::Idle)?;
    } else if original_ndm_wifi_state == 2 {
        svc::signal_event(&server.context.ndm_wifi_event_handle)?;
    }

    Ok(())
}

#[ctr_method(cmd = "FrdNCommand::GetWiFiState", normal = 0x2, translate = 0x0)]
fn get_wifi_state(server: &mut FriendSysmodule, _session_index: usize) -> CtrResult<u32> {
    Ok(wifi::get_wifi_state(
        server.context.ndm_wifi_state,
        server.context.wifi_connection_status,
    ))
}
