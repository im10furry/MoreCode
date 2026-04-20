use std::sync::OnceLock;

use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use aes_gcm::aead::Aead;
use base64::{Engine as _, engine::general_purpose};

static ENCRYPTION_KEY: OnceLock<[u8; 32]> = OnceLock::new();

pub fn init_encryption(key: &str) {
    let mut key_bytes = [0u8; 32];
    let key_len = key.as_bytes().len();
    key_bytes[..key_len.min(32)].copy_from_slice(&key.as_bytes()[..key_len.min(32)]);
    // Ignore error if key is already set
    let _ = ENCRYPTION_KEY.set(key_bytes);
}

pub fn encrypt(data: &str) -> Result<String, String> {
    let key = ENCRYPTION_KEY.get().ok_or("Encryption key not initialized")?;
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| e.to_string())?;
    
    let nonce = Nonce::from_slice(b"unique nonce");
    let ciphertext = cipher.encrypt(nonce, data.as_bytes()).map_err(|e| e.to_string())?;
    
    Ok(general_purpose::STANDARD.encode(ciphertext))
}

pub fn decrypt(encrypted: &str) -> Result<String, String> {
    let key = ENCRYPTION_KEY.get().ok_or("Encryption key not initialized")?;
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| e.to_string())?;
    
    let ciphertext = general_purpose::STANDARD.decode(encrypted).map_err(|e| e.to_string())?;
    let nonce = Nonce::from_slice(b"unique nonce");
    
    let plaintext = cipher.decrypt(nonce, ciphertext.as_ref()).map_err(|e| e.to_string())?;
    String::from_utf8(plaintext).map_err(|e| e.to_string())
}
