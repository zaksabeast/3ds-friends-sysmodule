use num_enum::IntoPrimitive;

#[derive(Debug, PartialEq, IntoPrimitive)]
#[repr(i32)]
pub enum FrdErrorCode {
    InvalidPointer = -0x1f1f380a,
    InvalidPrincipalId = -0x1f1f3b15,
    InvalidFriendCode = -0x1f1f3bff,
    InvalidErrorCode = -0x1f1f3bfd,
    InvalidFriendListOrMyDataSaveFile = -0x269f3b0c,
    InvalidArguments = -0x26ffe7d0,
    InvalidCommand = -0x26ffe7d1,
    InvalidAccountSaveFile = -0x377f3b13,
    MissingData = -0x375f3811,
}

impl From<FrdErrorCode> for u32 {
    fn from(error_code: FrdErrorCode) -> Self {
        error_code as u32
    }
}
