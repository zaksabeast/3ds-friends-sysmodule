use crate::frd::result::FrdErrorCode;

pub fn convert_principal_id_to_friend_code(principal_id: u32) -> Result<u64, FrdErrorCode> {
    if principal_id == 0 {
        return Err(FrdErrorCode::InvalidPrincipalId);
    }

    let mut hasher = sha1::Sha1::new();
    hasher.update(&principal_id.to_le_bytes());

    let hash = hasher.digest().bytes();
    let friend_code = (((hash[0] >> 1) as u64) << 32) | principal_id as u64;

    Ok(friend_code)
}

pub fn validate_friend_code(friend_code: u64) -> bool {
    if friend_code == 0 {
        return false;
    }

    let principal_id = friend_code as u32;

    match convert_principal_id_to_friend_code(principal_id) {
        Ok(test_friend_code) => friend_code == test_friend_code,
        Err(_) => false,
    }
}

pub fn convert_friend_code_to_principal_id(friend_code: u64) -> Result<u32, FrdErrorCode> {
    let is_valid = validate_friend_code(friend_code);

    if is_valid {
        Ok(friend_code as u32)
    } else {
        Err(FrdErrorCode::InvalidFriendCode)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod test_principal_id_to_friend_code {
        use super::*;

        #[test]
        fn should_return_friend_code() {
            let friend_code =
                convert_principal_id_to_friend_code(0xaabbccdd).expect("Expected friend code");
            assert_eq!(friend_code, 0x38aabbccdd);
        }

        #[test]
        fn should_return_error_code_if_principal_id_is_0() {
            let error_code =
                convert_principal_id_to_friend_code(0).expect_err("Expected error code");
            assert_eq!(error_code, FrdErrorCode::InvalidPrincipalId);
        }
    }

    mod test_validate_friend_code {
        use super::*;

        #[test]
        fn should_return_true_if_valid() {
            let is_valid = validate_friend_code(0x38aabbccdd);
            assert_eq!(is_valid, true);
        }

        #[test]
        fn should_return_false_if_invalid() {
            let is_valid = validate_friend_code(0x40aabbccdd);
            assert_eq!(is_valid, false);
        }

        #[test]
        fn should_return_false_if_given_zero() {
            let is_valid = validate_friend_code(0);
            assert_eq!(is_valid, false);
        }
    }

    mod test_convert_friend_code_to_principal_id {
        use super::*;

        #[test]
        fn should_return_principal_id_if_valid() {
            let principal_id =
                convert_friend_code_to_principal_id(0x38aabbccdd).expect("Expected principal Id");
            assert_eq!(principal_id, 0xaabbccdd);
        }

        #[test]
        fn should_return_error_code_if_invalid() {
            let error_code =
                convert_friend_code_to_principal_id(0x40aabbccdd).expect_err("Expected error code");
            assert_eq!(error_code, FrdErrorCode::InvalidFriendCode);
        }

        #[test]
        fn should_return_error_code_if_given_zero() {
            let error_code =
                convert_friend_code_to_principal_id(0).expect_err("Expected error code");
            assert_eq!(error_code, FrdErrorCode::InvalidFriendCode);
        }
    }
}
