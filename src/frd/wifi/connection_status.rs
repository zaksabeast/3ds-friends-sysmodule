#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum WiFiConnectionStatus {
    Idle = 0,
    Connecting = 1,
    Connected = 2,
    Disconnecting = 3,
}
