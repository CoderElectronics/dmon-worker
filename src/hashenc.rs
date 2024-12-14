#![allow(dead_code)]

use aes::cipher::BlockDecryptMut;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use cbc::cipher::block_padding::Pkcs7;
use cbc::cipher::{BlockEncryptMut, KeyIvInit};
use rand::RngCore;
use sha2::{Digest, Sha512};

pub fn generate_rand_iv() -> Result<[u8; 16], Box<dyn std::error::Error>> {
    let mut random_bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut random_bytes);

    // Create hasher and hash the random bytes
    let mut iv_hasher = Sha512::new();
    iv_hasher.update(&random_bytes);
    let iv_hash = iv_hasher.finalize();

    // Take first 16 bytes for IV
    let iv: [u8; 16] = iv_hash[0..16]
        .try_into()
        .map_err(|_| "Failed to convert IV hash to array")?;

    Ok(iv)
}

pub fn generate_base64_iv(iv: [u8; 16]) -> Result<String, Box<dyn std::error::Error>> {
    Ok(BASE64.encode(iv))
}

pub fn encrypt_payload(
    data: &str,
    key_string: &str,
    iv: [u8; 16],
) -> Result<String, Box<dyn std::error::Error>> {
    // Create SHA-512 hasher
    let mut key_hasher = Sha512::new();

    // Create hashes from input strings
    key_hasher.update(key_string.as_bytes());

    let key_hash = key_hasher.finalize();

    // Take first 32 bytes of key_hash for AES-256 key
    let key: [u8; 32] = key_hash[0..32]
        .try_into()
        .map_err(|_| "Failed to convert key hash to array")?;

    // Convert the input string to bytes
    let plaintext = data.as_bytes();

    // Create new cipher instance
    type Aes256Cbc = cbc::Encryptor<aes::Aes256>;

    // Create buffer for encrypted data
    let mut buffer = Vec::from(plaintext);
    // Ensure we have enough space for padding
    buffer.resize(buffer.len() + 16, 0);

    let cipher = Aes256Cbc::new_from_slices(&key, &iv).map_err(|_| "Failed to create cipher")?;

    // Encrypt in place
    let ciphertext_len = cipher
        .encrypt_padded_b2b_mut::<Pkcs7>(plaintext, &mut buffer)
        .map_err(|_| "Encryption failed")?
        .len();

    // Truncate to the actual ciphertext length
    buffer.truncate(ciphertext_len);

    // Convert to hex string
    Ok(hex::encode(buffer))
}

pub fn decrypt_payload(
    encrypted_hex: &str,
    key_string: &str,
    iv: [u8; 16],
) -> Result<String, Box<dyn std::error::Error>> {
    // Create SHA-512 hasher
    let mut key_hasher = Sha512::new();

    // Create hashes from input strings
    key_hasher.update(key_string.as_bytes());

    let key_hash = key_hasher.finalize();

    // Take first 32 bytes of key_hash for AES-256 key
    let key: [u8; 32] = key_hash[0..32]
        .try_into()
        .map_err(|_| "Failed to convert key hash to array")?;

    // Decode hex string to bytes
    let ciphertext = hex::decode(encrypted_hex).map_err(|_| "Failed to decode hex string")?;

    // Create new cipher instance
    type Aes256Cbc = cbc::Decryptor<aes::Aes256>;

    // Create buffer for decrypted data
    let mut buffer = Vec::from(&ciphertext[..]);

    let cipher = Aes256Cbc::new_from_slices(&key, &iv).map_err(|_| "Failed to create cipher")?;

    // Decrypt in place
    let plaintext =
        BlockDecryptMut::decrypt_padded_b2b_mut::<Pkcs7>(cipher, &ciphertext, &mut buffer)
            .map_err(|_| "Decryption failed")?;

    // Convert decrypted bytes to string
    String::from_utf8(plaintext.to_vec())
        .map_err(|_| "Failed to convert decrypted data to string".into())
}
