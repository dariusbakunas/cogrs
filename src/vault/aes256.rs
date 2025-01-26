use hmac::{Hmac, KeyInit, Mac};
use openssl::hash::MessageDigest;
use openssl::pkcs5::pbkdf2_hmac;
use openssl::symm::{Cipher, Crypter, Mode};
use sha2::Sha256;
use thiserror::Error;
use zeroize::Zeroize;

const SALT_LEN: usize = 32;
const KEY_LEN: usize = 32; // AES-256 and HMAC-SHA256 require 32-byte keys
const IV_LEN: usize = 16; // AES-CTR uses a 16-byte initialization vector
const PBKDF2_ITERATIONS: usize = 10_000;
const HMAC_LEN: usize = 32; // For SHA-256, the output size is always 32 bytes

pub struct HexUtils;

impl HexUtils {
    pub fn encode(data: &[u8]) -> String {
        data.iter().map(|b| format!("{:02x}", b)).collect()
    }

    pub fn decode(hex: &str) -> Result<Vec<u8>, AES256Error> {
        if hex.len() % 2 != 0 {
            return Err(AES256Error::InvalidHex(
                "Hex string has an invalid length. Must be even.".to_string(),
            ));
        }
        (0..hex.len())
            .step_by(2)
            .map(|i| {
                u8::from_str_radix(&hex[i..i + 2], 16).map_err(|_| {
                    AES256Error::InvalidHex(format!("Invalid hex pair: {}", &hex[i..i + 2]))
                })
            })
            .collect()
    }
}

pub struct KeyDeriver;

impl KeyDeriver {
    pub fn derive(secret: &[u8], salt: &[u8]) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), AES256Error> {
        let mut key_material = [0u8; KEY_LEN * 2 + IV_LEN];
        pbkdf2_hmac(
            secret,
            salt,
            PBKDF2_ITERATIONS,
            MessageDigest::sha256(),
            &mut key_material,
        )
        .map_err(|e| AES256Error::KeyDerivationFailed(e.to_string()))?;

        let (key1, rest) = key_material.split_at(KEY_LEN);
        let (key2, iv) = rest.split_at(KEY_LEN);

        let result = (key1.to_vec(), key2.to_vec(), iv.to_vec());
        let _ = key_material.zeroize();
        Ok(result)
    }
}

#[derive(Debug, Error)]
pub enum AES256Error {
    #[error("Invalid data format: {0}")]
    InvalidFormat(String),
    #[error("Invalid hex string: {0}")]
    InvalidHex(String),
    #[error("HMAC verification failed")]
    HmacFailure,
    #[error("Data integrity compromised: {0}")]
    IntegrityError(String),
    #[error("Encryption key derivation failed: {0}")]
    KeyDerivationFailed(String),
    #[error("Decryption produced invalid UTF-8 data")]
    InvalidUtf8Data,
    #[error("OpenSSL error: {0}")]
    OpenSslError(String),
    #[error("Random number generation error")]
    RngError,
}

impl From<openssl::error::ErrorStack> for AES256Error {
    fn from(err: openssl::error::ErrorStack) -> Self {
        AES256Error::OpenSslError(err.to_string())
    }
}

impl From<std::str::Utf8Error> for AES256Error {
    fn from(_: std::str::Utf8Error) -> Self {
        AES256Error::InvalidUtf8Data
    }
}

pub struct AES256;

impl AES256 {
    fn parse_encrypted_data(data: &str) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), AES256Error> {
        let encrypted_bytes = HexUtils::decode(data)?;
        if encrypted_bytes.len() < SALT_LEN + HMAC_LEN + 1 {
            Err(AES256Error::InvalidFormat(
                "Encrypted data is too short to contain salt, HMAC, and ciphertext".to_string(),
            ))?
        }

        let parts: Vec<_> = encrypted_bytes.split(|&b| b == b'\n').collect();
        if parts.len() != 3 {
            Err(AES256Error::InvalidFormat(
                "Encrypted data is missing salt, hmac_tag or ciphertext".to_string(),
            ))?;
        }

        let salt = HexUtils::decode(std::str::from_utf8(parts[0])?)?;
        let hmac_tag = HexUtils::decode(std::str::from_utf8(parts[1])?)?;
        let ciphertext = HexUtils::decode(std::str::from_utf8(parts[2])?)?;

        Ok((salt, hmac_tag, ciphertext))
    }

    pub fn decrypt_aes256(data: &str, secret: &str) -> Result<String, AES256Error> {
        // Decode hex-encoded data
        let (salt, crypted_hmac, ciphertext) = AES256::parse_encrypted_data(data)?;
        let (key1, key2, iv) = KeyDeriver::derive(secret.as_bytes(), salt.as_slice())?;

        // Verify HMAC-SHA256 tag
        let mut hmac = Hmac::<Sha256>::new_from_slice(key2.as_slice())
            .map_err(|_| AES256Error::HmacFailure)?;
        hmac.update(ciphertext.as_slice()); // Include ciphertext in HMAC
        hmac.verify_slice(crypted_hmac.as_slice())
            .map_err(|_| AES256Error::IntegrityError("Failed to verify HMAC tag".to_string()))?;

        let plaintext = AES256::crypt_aes256_ctr(Mode::Decrypt, &key1, &iv, &ciphertext)?;

        // Convert to string
        let plaintext_str =
            String::from_utf8(plaintext).map_err(|_| AES256Error::InvalidUtf8Data)?;

        Ok(plaintext_str)
    }

    fn crypt_aes256_ctr(
        mode: Mode,
        key: &[u8],
        iv: &[u8],
        input: &[u8],
    ) -> anyhow::Result<Vec<u8>, AES256Error> {
        let cipher = Cipher::aes_256_ctr();
        let mut crypter = Crypter::new(cipher, mode, key, Some(iv))?;
        crypter.pad(false);

        let mut output = vec![0; input.len() + cipher.block_size()];
        let mut count = crypter.update(input, &mut output)?;
        count += crypter.finalize(&mut output[count..])?;
        output.truncate(count);
        Ok(output)
    }

    pub fn encrypt_aes256(data: &str, secret: &str) -> Result<String, AES256Error> {
        use openssl::rand::rand_bytes;

        // Generate a random salt and initialization vector (IV)
        let mut salt = [0u8; SALT_LEN];
        rand_bytes(&mut salt).map_err(|_| AES256Error::RngError)?;

        let (key1, key2, iv) = KeyDeriver::derive(secret.as_bytes(), salt.as_slice())?;
        let ciphertext = AES256::crypt_aes256_ctr(Mode::Encrypt, &key1, &iv, data.as_bytes())?;

        // Compute HMAC-SHA256 for the ciphertext
        let mut hmac = Hmac::<Sha256>::new_from_slice(key2.as_slice())
            .map_err(|_| AES256Error::HmacFailure)?;
        hmac.update(&ciphertext);
        let hmac_tag = hmac.finalize().into_bytes();

        // Combine salt, HMAC, and ciphertext, separated by newlines
        let result = format!(
            "{}\n{}\n{}",
            HexUtils::encode(&salt),
            HexUtils::encode(&hmac_tag),
            HexUtils::encode(&ciphertext)
        );

        Ok(HexUtils::encode(result.into_bytes().as_slice()))
    }
}

#[cfg(test)]
mod tests {
    use super::{AES256Error, HexUtils, KeyDeriver, AES256};
    use openssl::rand::rand_bytes;

    const TEST_SECRET: &str = "my_secure_password";
    const TEST_DATA: &str = "This is a test message!";

    #[test]
    fn test_encrypt_and_decrypt_success() {
        let encrypted = AES256::encrypt_aes256(TEST_DATA, TEST_SECRET).expect("Encryption failed");

        let decrypted = AES256::decrypt_aes256(&encrypted, TEST_SECRET).expect("Decryption failed");

        assert_eq!(decrypted, TEST_DATA);
    }

    #[test]
    fn test_encrypt_and_decrypt_with_different_secrets_fails() {
        let encrypted = AES256::encrypt_aes256(TEST_DATA, TEST_SECRET).expect("Encryption failed");

        let decryption_result = AES256::decrypt_aes256(&encrypted, "wrong_password");

        assert!(decryption_result.is_err());
        if let Err(AES256Error::IntegrityError(msg)) = decryption_result {
            assert!(msg.contains("Failed to verify HMAC tag"));
        } else {
            panic!("Expected an IntegrityError, got {:?}", decryption_result);
        }
    }

    #[test]
    fn test_invalid_encrypted_format_fails() {
        let invalid_data = "invalid_hex_string";
        let encoded_data = HexUtils::encode(invalid_data.as_bytes());

        let decryption_result = AES256::decrypt_aes256(&encoded_data, TEST_SECRET);

        assert!(decryption_result.is_err());
        if let Err(AES256Error::InvalidFormat(msg)) = decryption_result {
            assert!(msg.contains("Encrypted data is too short"));
        } else {
            panic!(
                "Expected an InvalidFormat error, got {:?}",
                decryption_result
            );
        }
    }

    #[test]
    fn test_key_derivation_works() {
        let secret = TEST_SECRET.as_bytes();
        let mut salt = [0u8; 32];
        rand_bytes(&mut salt).expect("Failed to generate random salt");

        let result = KeyDeriver::derive(secret, &salt).expect("Key derivation failed");

        assert_eq!(result.0.len(), 32, "Key1 should be 32 bytes");
        assert_eq!(result.1.len(), 32, "Key2 should be 32 bytes");
        assert_eq!(result.2.len(), 16, "IV should be 16 bytes");
    }

    #[test]
    fn test_hex_encoding_and_decoding() {
        let data = [1, 2, 3, 255];
        let encoded = HexUtils::encode(&data);
        assert_eq!(encoded, "010203ff");

        let decoded = HexUtils::decode(&encoded).expect("Failed to decode hex string");
        assert_eq!(decoded, data);

        // Test invalid hex decoding
        let invalid_hex = "01GA"; // Contains non-hexadecimal character 'G'
        let decode_result = HexUtils::decode(invalid_hex);

        assert!(decode_result.is_err());
        if let Err(AES256Error::InvalidHex(msg)) = decode_result {
            assert!(msg.contains("Invalid hex pair: GA"));
        } else {
            panic!("Expected an InvalidHex error, got {:?}", decode_result);
        }
    }

    #[test]
    #[should_panic(expected = "Failed to verify HMAC tag")]
    fn test_hmac_verification_failure() {
        let encrypted = AES256::encrypt_aes256(TEST_DATA, TEST_SECRET).expect("Encryption failed");

        // Tamper with the HMAC section specifically
        let mut tampered_encrypted = encrypted.clone();
        let (salt, hmac, ciphertext) = AES256::parse_encrypted_data(&tampered_encrypted).unwrap();

        let mut tampered_hmac = hmac.clone(); // Get the HMAC part
        if let Some(pos) = tampered_hmac.len().checked_sub(2) {
            tampered_hmac.insert(pos, 255); // Tamper with valid hex in the HMAC part
        }
        tampered_encrypted = format!(
            "{}\n{}\n{}",
            HexUtils::encode(&salt),
            HexUtils::encode(&tampered_hmac),
            HexUtils::encode(&ciphertext)
        ); // Reconstruct tampered data
        tampered_encrypted = HexUtils::encode(tampered_encrypted.as_bytes());

        // This should fail due to HMAC mismatch
        let _ = AES256::decrypt_aes256(&tampered_encrypted, TEST_SECRET).unwrap();
        // Should panic because of HMAC failure
    }

    #[test]
    fn test_encrypt_and_decrypt_empty_string() {
        let empty_string = "";

        let encrypted =
            AES256::encrypt_aes256(empty_string, TEST_SECRET).expect("Encryption failed");

        let decrypted = AES256::decrypt_aes256(&encrypted, TEST_SECRET).expect("Decryption failed");

        assert_eq!(decrypted, empty_string);
    }
}
