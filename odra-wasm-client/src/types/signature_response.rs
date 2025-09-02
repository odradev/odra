use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct SignatureResponse {
    cancelled: bool,
    #[serde(rename = "signatureHex")]
    signature_hex: Option<String>,
    signature: Option<HashMap<String, u8>>
}

impl SignatureResponse {
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }

    pub fn get_signature_hex(&self) -> String {
        self.signature_hex.clone().unwrap_or_default()
    }

    pub fn get_signature(&self) -> Vec<u8> {
        let signature = self.signature.clone().unwrap_or_default();
        let mut signature_vec = vec![0; signature.len()];
        for (key, value) in &signature {
            if let Ok(index) = key.parse::<usize>() {
                if index < signature_vec.len() {
                    signature_vec[index] = *value;
                }
            }
        }
        signature_vec
    }
}
