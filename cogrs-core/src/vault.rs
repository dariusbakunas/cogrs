pub mod aes256;

use crate::constants::VAULT_HEADER;
use crate::vault::aes256::AES256;
use anyhow::{bail, Result};

#[derive(Default, Debug)]
pub struct Vault {}

impl Vault {
    pub fn new() -> Self {
        Vault {}
    }

    pub fn is_encrypted(&self, data: &str) -> bool {
        data.starts_with(VAULT_HEADER) && data.is_ascii()
    }

    fn parse_vaulttext_envelope(
        &self,
        vaulttext: &str,
    ) -> Result<(String, String, String, String, Option<String>)> {
        let mut lines = vaulttext.lines();
        let first_line = match lines.next() {
            Some(line) => line,
            None => bail!("Vault text is empty, unable to parse envelope"),
        };

        let parts: Vec<&str> = first_line.split(';').collect();

        if parts.len() < 3 {
            bail!("Invalid vault text header format");
        }

        let header = parts[0].to_string();
        let version = parts[1].to_string();
        let cipher = parts[2].to_string();

        let mut vault_id = None;

        if version == "1.2" {
            if parts.len() < 4 {
                bail!("Version 1.2 requires a vault_id, but it is missing");
            }
            vault_id = Some(parts[3].to_string());
        }

        let remaining_text = lines.collect::<Vec<&str>>().join("\n");

        Ok((remaining_text, header, version, cipher, vault_id))
    }

    pub fn decrypt(&self, data: &str, secret: &str) -> Result<String> {
        let (vault_text, _header, _version, cipher, _vault_id) =
            self.parse_vaulttext_envelope(data)?;

        let plain_text = match cipher.as_str() {
            "AES256" => AES256::decrypt_aes256(&vault_text, secret),
            _ => bail!("Unsupported cipher: {}", cipher),
        }?;

        Ok(plain_text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vault::aes256::AES256;

    fn encrypt_test_data(secret: &str, data: &str) -> Result<String> {
        let encoded_data = AES256::encrypt_aes256(data, secret)?;

        // Format as a valid vault string
        Ok(format!("$ANSIBLE_VAULT;1.1;AES256\n{}", encoded_data))
    }

    #[test]
    fn test_success_decrypt() {
        let vault = Vault::new();

        // Encrypt a test payload
        let secret = "test_secret";
        let plaintext = "Hello, Vault!";
        let encrypted_vault_text = encrypt_test_data(secret, plaintext).unwrap();

        // Decrypt the payload
        let result = vault.decrypt(&encrypted_vault_text, secret);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), plaintext);
    }

    #[test]
    fn test_invalid_cipher() {
        let vault = Vault::new();

        // Simulate an invalid cipher by modifying the header
        let invalid_vault_text = "$ANSIBLE_VAULT;1.1;INVALID_CIPHER\nSGVsbG8gVmF1bHQ=";

        let result = vault.decrypt(invalid_vault_text, "test_secret");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Unsupported cipher: INVALID_CIPHER"
        );
    }

    #[test]
    fn test_missing_header() {
        let vault = Vault::new();

        // Input doesn't have a valid header
        let invalid_vault_text = "SGVsbG8gVmF1bHQ=";

        let result = vault.decrypt(invalid_vault_text, "test_secret");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid vault text header format"
        );
    }

    #[test]
    fn test_invalid_base64_encoding() {
        let vault = Vault::new();

        // Simulate invalid base64 encoding of the vault text
        let invalid_vault_text = "$ANSIBLE_VAULT;1.1;AES256\nInvalidBase64==";

        let result = vault.decrypt(invalid_vault_text, "test_secret");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid hex string: Hex string has an invalid length. Must be even."
        );
    }

    #[test]
    fn test_decryption_with_wrong_secret() {
        let vault = Vault::new();

        // Encrypt a test payload with a specific secret
        let secret = "test_secret";
        let plaintext = "Hello, Vault!";
        let encrypted_vault_text = encrypt_test_data(secret, plaintext).unwrap();

        // Try decrypting with a wrong secret
        let wrong_secret = "wrong_secret";
        let result = vault.decrypt(&encrypted_vault_text, wrong_secret);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Data integrity compromised: Failed to verify HMAC tag"
        );
    }
}
