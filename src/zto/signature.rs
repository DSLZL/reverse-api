use sha2::{Digest, Sha256};

pub fn generate_signature(body: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(body);
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_generation() {
        let test_data = b"test";
        let sig = generate_signature(test_data);
        assert!(!sig.is_empty());
        assert_eq!(sig.len(), 64);
    }
}
