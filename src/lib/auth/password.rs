use password_auth::{generate_hash, verify_password as verify_hash, VerifyError};

/// Hash a password using Argon2id
pub fn hash_password(password: &str) -> Result<String, String> {
    let hash = generate_hash(password);
    Ok(hash)
}

/// Verify a password against its hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool, VerifyError> {
    match verify_hash(password, hash) {
        Ok(_) => Ok(true),
        Err(VerifyError::PasswordInvalid) => Ok(false),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hash_and_verify() {
        let password = "SecurePassword123!";
        let hash = hash_password(password).unwrap();

        // Verify correct password
        assert!(verify_password(password, &hash).unwrap());

        // Verify incorrect password
        assert!(!verify_password("WrongPassword", &hash).unwrap());
    }

    #[test]
    fn test_different_hashes() {
        let password = "SecurePassword123!";
        let hash1 = hash_password(password).unwrap();
        let hash2 = hash_password(password).unwrap();

        // Same password should produce different hashes (due to salt)
        assert_ne!(hash1, hash2);

        // But both should verify successfully
        assert!(verify_password(password, &hash1).unwrap());
        assert!(verify_password(password, &hash2).unwrap());
    }
}
