use std::error::Error;
use aes::Aes128;
use block_modes::block_padding::Pkcs7;
use block_modes::{BlockMode, Ecb};
use hex::{decode, encode};
use crate::configuration::Processor;
use hex_literal::hex;
use base64;

type Aes128Ecb = Ecb<Aes128, Pkcs7>;

pub struct AesEncDec {
    key: Vec<u8>,
}

impl AesEncDec {
    pub fn new(key: &str) -> AesEncDec {
        let mut key_bytes = key.as_bytes().to_vec();
        key_bytes.resize(16, 0);
        AesEncDec { key: key_bytes }
    }
}

impl Processor for AesEncDec {
    fn before_save(&self, content: String) -> Result<String, Box<dyn Error>> {
        if content.is_empty() {
            return Ok(content.to_string());
        }
        let plaintext= content.as_bytes();
        let cipher = Aes128Ecb::new_from_slices(&self.key, Default::default()).unwrap();
        let pos = plaintext.len();
        let mut buffer = [0u8; 128];
        buffer[..pos].copy_from_slice(plaintext);
        let encrypted_data = cipher.encrypt(&mut buffer, pos).unwrap();
        // Convert encrypted bytes to Base64
        let encrypted_base64 = base64::encode(&encrypted_data);
        Ok(encrypted_base64)
    }

    fn after_fetch(&self, content: String) -> Result<String, Box<dyn Error>> {
        if content.is_empty() {
            return Ok(content.to_string());
        }
        let cipher = Aes128Ecb::new_from_slices(&self.key, Default::default())?;
        let encrypted_data = decode(content)?;
        let decrypted_data = cipher.decrypt_vec(&encrypted_data)?;

        String::from_utf8(decrypted_data).map_err(|e| e.into())
    }
}