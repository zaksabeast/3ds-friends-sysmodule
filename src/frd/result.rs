use ctr::result::ResultCode;
use num_enum::IntoPrimitive;

// TODO: Replace these with proper ctr::result::ResultCodes.
#[derive(Debug, PartialEq, Eq, IntoPrimitive)]
#[repr(u32)]
pub enum FrdErrorCode {
    InvalidPointer = 0xe0e0c7f6,
    InvalidPrincipalId = 0xe0e0c4eb,
    InvalidFriendCode = 0xe0e0c401,
    InvalidErrorCode = 0xe0e0c403,
    InvalidFriendListOrMyDataSaveFile = 0xd960c4f4,
    InvalidArguments = 0xd9001830,
    InvalidCommand = 0xd900182f,
    InvalidAccountSaveFile = 0xc880c4ed,
    MissingData = 0xc8a0c7ef,
}

impl FrdErrorCode {
    // Convenience method for ambiguious convertions
    pub fn into_result_code(self) -> ResultCode {
        self.into()
    }
}

impl From<FrdErrorCode> for ResultCode {
    fn from(result_code: FrdErrorCode) -> Self {
        ResultCode::new_from_raw(result_code.into())
    }
}
