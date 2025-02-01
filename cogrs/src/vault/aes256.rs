use aes::Aes256;
use cipher::{KeyIvInit, StreamCipher};
use ctr::Ctr128BE;
use hmac::{Hmac, KeyInit, Mac};
use rand::rngs::OsRng;
use rand::TryRngCore;
use ring::pbkdf2;
use sha2::Sha256;
use std::fmt::Write;
use std::num::NonZeroU32;
use thiserror::Error;
use zeroize::Zeroize;

const SALT_LEN: usize = 32;
const KEY_LEN: usize = 32; // AES-256 and HMAC-SHA256 require 32-byte keys
const IV_LEN: usize = 16; // AES-CTR uses a 16-byte initialization vector
const PBKDF2_ITERATIONS: u32 = 10_000;
const HMAC_LEN: usize = 32; // For SHA-256, the output size is always 32 bytes

pub struct HexUtils;

impl HexUtils {
    pub fn encode(data: &[u8]) -> String {
        // using write! instead of format! to avoid extra allocations
        data.iter().fold(String::new(), |mut acc, b| {
            let _ = write!(acc, "{:02x}", b);
            acc
        })
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

pub struct DerivedKeys {
    pub key1: [u8; KEY_LEN],
    pub key2: [u8; KEY_LEN],
    pub iv: [u8; IV_LEN],
}

impl KeyDeriver {
    pub fn derive(secret: &[u8], salt: &[u8]) -> Result<DerivedKeys, AES256Error> {
        let mut key_material = [0u8; KEY_LEN * 2 + IV_LEN];

        // Derive key material using PBKDF2
        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA256,
            NonZeroU32::new(PBKDF2_ITERATIONS).unwrap(),
            salt,
            secret,
            &mut key_material,
        );

        let (key1, rest) = key_material.split_at(KEY_LEN);
        let (key2, iv) = rest.split_at(KEY_LEN);

        let result = DerivedKeys {
            key1: <[u8; KEY_LEN]>::try_from(key1).unwrap(),
            key2: <[u8; KEY_LEN]>::try_from(key2).unwrap(),
            iv: <[u8; IV_LEN]>::try_from(iv).unwrap(),
        };

        key_material.zeroize();
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

impl From<std::str::Utf8Error> for AES256Error {
    fn from(_: std::str::Utf8Error) -> Self {
        AES256Error::InvalidUtf8Data
    }
}

pub struct ParsedEncryptedData {
    pub salt: Vec<u8>,
    pub hmac_tag: Vec<u8>,
    pub ciphertext: Vec<u8>,
}

pub struct AES256;
pub type Aes256Ctr = Ctr128BE<Aes256>;

impl AES256 {
    fn parse_encrypted_data(data: &str) -> Result<ParsedEncryptedData, AES256Error> {
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

        Ok(ParsedEncryptedData {
            salt,
            hmac_tag,
            ciphertext,
        })
    }

    pub fn decrypt_aes256(data: &str, secret: &str) -> Result<String, AES256Error> {
        // Decode hex-encoded data
        let parsed_encrypted_data = AES256::parse_encrypted_data(data)?;
        let derived_keys =
            KeyDeriver::derive(secret.as_bytes(), parsed_encrypted_data.salt.as_slice())?;

        // Verify HMAC-SHA256 tag
        let mut hmac = Hmac::<Sha256>::new_from_slice(derived_keys.key2.as_slice())
            .map_err(|_| AES256Error::HmacFailure)?;
        hmac.update(parsed_encrypted_data.ciphertext.as_slice()); // Include ciphertext in HMAC
        hmac.verify_slice(parsed_encrypted_data.hmac_tag.as_slice())
            .map_err(|_| AES256Error::IntegrityError("Failed to verify HMAC tag".to_string()))?;

        let mut cipher = Aes256Ctr::new(derived_keys.key1.as_ref(), derived_keys.iv.as_ref());

        let mut plaintext = parsed_encrypted_data.ciphertext.to_vec();
        cipher.apply_keystream(&mut plaintext);

        // Convert to string
        String::from_utf8(plaintext).map_err(|_| AES256Error::InvalidUtf8Data)
    }

    pub fn encrypt_aes256(data: &str, secret: &str) -> Result<String, AES256Error> {
        let mut salt = [0u8; SALT_LEN];
        OsRng
            .try_fill_bytes(&mut salt)
            .or(Err(AES256Error::RngError))?;

        let derived_keys = KeyDeriver::derive(secret.as_bytes(), salt.as_slice())?;

        let mut cipher = Aes256Ctr::new((&derived_keys.key1).into(), (&derived_keys.iv).into());

        let mut ciphertext = data.as_bytes().to_vec();
        cipher.apply_keystream(&mut ciphertext);

        // Compute HMAC-SHA256 for the ciphertext
        let mut hmac = Hmac::<Sha256>::new_from_slice(derived_keys.key2.as_slice())
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
    use rand::rngs::OsRng;
    use rand::TryRngCore;

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
        OsRng
            .try_fill_bytes(&mut salt)
            .or(Err(AES256Error::RngError))
            .unwrap();

        let derived_keys = KeyDeriver::derive(secret, &salt).expect("Key derivation failed");

        assert_eq!(derived_keys.key1.len(), 32, "Key1 should be 32 bytes");
        assert_eq!(derived_keys.key2.len(), 32, "Key2 should be 32 bytes");
        assert_eq!(derived_keys.iv.len(), 16, "IV should be 16 bytes");
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
        let parsed_encrypted_data = AES256::parse_encrypted_data(&tampered_encrypted).unwrap();

        let mut tampered_hmac = parsed_encrypted_data.hmac_tag.clone(); // Get the HMAC part
        if let Some(pos) = tampered_hmac.len().checked_sub(2) {
            tampered_hmac.insert(pos, 255); // Tamper with valid hex in the HMAC part
        }
        tampered_encrypted = format!(
            "{}\n{}\n{}",
            HexUtils::encode(&parsed_encrypted_data.salt),
            HexUtils::encode(&tampered_hmac),
            HexUtils::encode(&parsed_encrypted_data.ciphertext)
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
