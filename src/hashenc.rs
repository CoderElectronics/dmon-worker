use aes::cipher::BlockDecryptMut;
use cbc::cipher::block_padding::Pkcs7;
use cbc::cipher::{BlockEncryptMut, KeyIvInit};
use sha2::{Digest, Sha512};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn decrypt_payload_ffi(
    encrypted_hex: *const c_char,
    key_string: *const c_char,
    iv_string: *const c_char,
) -> *mut c_char {
    // Convert C strings to Rust strings
    let c_enc = unsafe { CStr::from_ptr(encrypted_hex) };
    let c_key = unsafe { CStr::from_ptr(key_string) };
    let c_iv = unsafe { CStr::from_ptr(iv_string) };

    let rust_enc = c_enc.to_str().unwrap();
    let rust_key = c_key.to_str().unwrap();
    let rust_iv = c_iv.to_str().unwrap();

    // Process the strings (concatenate and uppercase in this example)
    let processed = match decrypt_payload(rust_enc, rust_key, rust_iv) {
        Ok(s) => s,
        Err(_e) => "".to_string(),
    };

    // Convert back to C string and return
    let c_string = CString::new(processed).unwrap();
    c_string.into_raw() // Transfer ownership to caller
}

#[no_mangle]
pub extern "C" fn encrypt_payload_ffi(
    data: *const c_char,
    key_string: *const c_char,
    iv_string: *const c_char,
) -> *mut c_char {
    // Convert C strings to Rust strings
    let c_data = unsafe { CStr::from_ptr(data) };
    let c_key = unsafe { CStr::from_ptr(key_string) };
    let c_iv = unsafe { CStr::from_ptr(iv_string) };

    let rust_data = c_data.to_str().unwrap();
    let rust_key = c_key.to_str().unwrap();
    let rust_iv = c_iv.to_str().unwrap();

    // Process the strings (concatenate and uppercase in this example)
    let processed = match encrypt_payload(rust_data, rust_key, rust_iv) {
        Ok(s) => s,
        Err(_e) => "".to_string(),
    };

    // Convert back to C string and return
    let c_string = CString::new(processed).unwrap();
    c_string.into_raw() // Transfer ownership to caller
}

#[no_mangle]
pub extern "C" fn free_string(ptr: *mut c_char) {
    unsafe {
        if !ptr.is_null() {
            let _ = CString::from_raw(ptr);
        }
    }
}

pub fn encrypt_payload(
    data: &str,
    key_string: &str,
    iv_string: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    // Create SHA-512 hasher
    let mut key_hasher = Sha512::new();
    let mut iv_hasher = Sha512::new();

    // Create hashes from input strings
    key_hasher.update(key_string.as_bytes());
    iv_hasher.update(iv_string.as_bytes());

    let key_hash = key_hasher.finalize();
    let iv_hash = iv_hasher.finalize();

    // Take first 32 bytes of key_hash for AES-256 key
    let key: [u8; 32] = key_hash[0..32]
        .try_into()
        .map_err(|_| "Failed to convert key hash to array")?;

    // Take first 16 bytes of iv_hash for IV
    let iv: [u8; 16] = iv_hash[0..16]
        .try_into()
        .map_err(|_| "Failed to convert IV hash to array")?;

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
    iv_string: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    // Create SHA-512 hasher
    let mut key_hasher = Sha512::new();
    let mut iv_hasher = Sha512::new();

    // Create hashes from input strings
    key_hasher.update(key_string.as_bytes());
    iv_hasher.update(iv_string.as_bytes());

    let key_hash = key_hasher.finalize();
    let iv_hash = iv_hasher.finalize();

    // Take first 32 bytes of key_hash for AES-256 key
    let key: [u8; 32] = key_hash[0..32]
        .try_into()
        .map_err(|_| "Failed to convert key hash to array")?;

    // Take first 16 bytes of iv_hash for IV
    let iv: [u8; 16] = iv_hash[0..16]
        .try_into()
        .map_err(|_| "Failed to convert IV hash to array")?;

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
