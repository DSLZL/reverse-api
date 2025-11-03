use crate::grok::error::{GrokError, Result};
use base64::{engine::general_purpose, Engine as _};
use rand::RngCore;
use secp256k1::{Message, Secp256k1, SecretKey};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

pub struct Anon;

impl Anon {
    /// Generate keypair for anonymous authentication
    pub fn generate_keys() -> Result<HashMap<String, Vec<u8>>> {
        let secp = Secp256k1::new();
        let mut rng = rand::thread_rng();

        // Generate 32 random bytes for private key
        let mut private_key_bytes = [0u8; 32];
        rng.fill_bytes(&mut private_key_bytes);

        // Create secret key
        let secret_key = SecretKey::from_slice(&private_key_bytes)
            .map_err(|e| GrokError::CryptoError(e.to_string()))?;

        // Derive public key (compressed format)
        let public_key = secp256k1::PublicKey::from_secret_key(&secp, &secret_key);
        let public_key_bytes = public_key.serialize().to_vec();

        let mut keys = HashMap::new();
        keys.insert("privateKey".to_string(), private_key_bytes.to_vec());
        keys.insert("userPublicKey".to_string(), public_key_bytes);

        Ok(keys)
    }

    /// Sign challenge for authentication
    pub fn sign_challenge(
        challenge_data: &[u8],
        private_key_b64: &str,
    ) -> Result<HashMap<String, String>> {
        // Decode private key from base64
        let private_key_bytes = general_purpose::STANDARD.decode(private_key_b64)?;

        let secret_key = SecretKey::from_slice(&private_key_bytes)
            .map_err(|e| GrokError::CryptoError(e.to_string()))?;

        // Hash the challenge
        let mut hasher = Sha256::new();
        hasher.update(challenge_data);
        let hash = hasher.finalize();

        let message =
            Message::from_digest_slice(&hash).map_err(|e| GrokError::CryptoError(e.to_string()))?;

        let secp = Secp256k1::new();
        let signature = secp.sign_ecdsa(&message, &secret_key);
        let sig_bytes = signature.serialize_compact();

        let mut result = HashMap::new();
        result.insert(
            "challenge".to_string(),
            general_purpose::STANDARD.encode(challenge_data),
        );
        result.insert(
            "signature".to_string(),
            general_purpose::STANDARD.encode(&sig_bytes[..64]),
        );

        Ok(result)
    }

    /// Encode bytes to base64 (XOR pattern from Python)
    pub fn xor_encode(bytes: &[u8]) -> String {
        general_purpose::STANDARD.encode(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_keys() {
        let keys = Anon::generate_keys().unwrap();
        assert!(keys.contains_key("privateKey"));
        assert!(keys.contains_key("userPublicKey"));
        assert_eq!(keys["privateKey"].len(), 32);
        assert_eq!(keys["userPublicKey"].len(), 33); // Compressed public key
    }

    #[test]
    fn test_xor_encode() {
        let bytes = vec![1, 2, 3, 4];
        let encoded = Anon::xor_encode(&bytes);
        assert!(!encoded.is_empty());
    }
}
