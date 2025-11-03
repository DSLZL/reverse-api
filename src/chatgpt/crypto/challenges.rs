use crate::chatgpt::utils::Result;
use base64::{engine::general_purpose, Engine as _};
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};

/// Challenge solver for proof-of-work and token generation
pub struct Challenges;

impl Challenges {
    /// Encode configuration to base64 JSON
    pub fn encode(config: &Value) -> Result<String> {
        let json_str = serde_json::to_string(config)?;
        let encoded = general_purpose::STANDARD.encode(json_str.as_bytes());
        Ok(encoded)
    }

    /// Generate token from configuration
    pub fn generate_token(mut config: Value) -> Result<String> {
        // Update config values
        if let Some(config_array) = config.as_array_mut() {
            if config_array.len() > 3 {
                config_array[3] = Value::Number(serde_json::Number::from(1));
            }
            if config_array.len() > 9 {
                config_array[9] = Value::Number(
                    serde_json::Number::from_f64(0.0)
                        .unwrap_or_else(|| serde_json::Number::from(0)),
                );
            }
        }

        let encoded = Self::encode(&config)?;
        Ok(format!("gAAAAAC{}", encoded))
    }

    /// Hash function similar to Python's mod function
    pub fn hash_mod(input: &str) -> String {
        let mut hash: u32 = 2166136261;

        for byte in input.bytes() {
            hash ^= byte as u32;
            hash = hash.wrapping_mul(16777619);
        }

        hash ^= hash >> 16;
        hash = hash.wrapping_mul(2246822507);
        hash ^= hash >> 13;
        hash = hash.wrapping_mul(3266489909);
        hash ^= hash >> 16;

        format!("{:08x}", hash)
    }

    /// Check if solution is valid
    fn run_check(
        start_time: u128,
        seed: &str,
        difficulty: &str,
        nonce: i32,
        mut config: Value,
    ) -> Option<String> {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        // Update config
        if let Some(config_array) = config.as_array_mut() {
            if config_array.len() > 3 {
                config_array[3] = Value::Number(serde_json::Number::from(nonce));
            }
            if config_array.len() > 9 {
                let elapsed = current_time - start_time;
                config_array[9] = Value::Number(
                    serde_json::Number::from_f64(elapsed as f64)
                        .unwrap_or(serde_json::Number::from(0)),
                );
            }
        }

        let encoded = Self::encode(&config).ok()?;
        let hash_input = format!("{}{}", seed, encoded);
        let hash_result = Self::hash_mod(&hash_input);

        if hash_result.len() >= difficulty.len() && &hash_result[..difficulty.len()] <= difficulty {
            Some(format!("{}~S", encoded))
        } else {
            None
        }
    }

    /// Solve proof-of-work challenge
    pub fn solve_pow(seed: &str, difficulty: &str, config: Value) -> Result<String> {
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        for nonce in 0..500_000 {
            if let Some(solution) =
                Self::run_check(start_time, seed, difficulty, nonce, config.clone())
            {
                return Ok(format!("gAAAAAB{}", solution));
            }
        }

        Err(crate::chatgpt::utils::ChatGptError::challenge_solve(
            "Failed to solve proof-of-work within iteration limit",
        ))
    }
}
