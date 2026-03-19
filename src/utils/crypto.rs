// src/utils/crypto.rs
use std::error::Error;
use tracing::warn;

/// Verify an Ethereum signature
/// 
/// This is a placeholder implementation for development/testing.
/// In production, you MUST implement proper Ethereum signature verification
/// using a library like ethers-rs or web3.
pub fn verify_ethereum_signature(
    signature: &str,
    message: &str,
    expected_address: &str,
) -> Result<bool, Box<dyn Error>> {
    // Basic format validation
    if signature.len() != 132 || !signature.starts_with("0x") {
        return Ok(false);
    }
    
    if expected_address.len() != 42 || !expected_address.starts_with("0x") {
        return Ok(false);
    }
    
    if message.is_empty() {
        return Ok(false);
    }
    
    // TODO: Implement proper Ethereum signature verification
    // 
    // Steps for proper implementation:
    // 1. Parse the signature into r, s, v components
    // 2. Hash the message with Ethereum's message signing prefix:
    //    "\x19Ethereum Signed Message:\n" + message.length + message
    // 3. Recover the public key from the signature and message hash
    // 4. Derive the Ethereum address from the public key
    // 5. Compare with the expected address
    //
    // Example using ethers-rs:
    // ```rust
    // use ethers::core::{
    //     types::{Address, Signature},
    //     utils::hash_message,
    // };
    // 
    // let signature: Signature = signature.parse()?;
    // let message_hash = hash_message(message);
    // let recovered_address = signature.recover(message_hash)?;
    // let expected: Address = expected_address.parse()?;
    // 
    // Ok(recovered_address == expected)
    // ```
    
    warn!("Using placeholder signature verification - IMPLEMENT PROPER VERIFICATION FOR PRODUCTION");
    
    // For development/testing only - always returns true
    // This allows wallet login to work during development
    Ok(true)
}

/// Hash a message with Ethereum's message signing prefix
pub fn hash_message_ethereum(message: &str) -> Vec<u8> {
    use sha3::{Digest, Keccak256};
    
    let prefix = format!("\x19Ethereum Signed Message:\n{}", message.len());
    let mut hasher = Keccak256::new();
    hasher.update(prefix.as_bytes());
    hasher.update(message.as_bytes());
    hasher.finalize().to_vec()
}

/// Parse an Ethereum signature into r, s, v components
pub fn parse_signature(signature: &str) -> Result<(Vec<u8>, Vec<u8>, u8), Box<dyn Error>> {
    if signature.len() != 132 || !signature.starts_with("0x") {
        return Err("Invalid signature format".into());
    }
    
    let sig_bytes = hex::decode(&signature[2..])?;
    
    if sig_bytes.len() != 65 {
        return Err("Invalid signature length".into());
    }
    
    let r = sig_bytes[0..32].to_vec();
    let s = sig_bytes[32..64].to_vec();
    let v = sig_bytes[64];
    
    Ok((r, s, v))
}

/// Validate Ethereum address format
pub fn is_valid_ethereum_address(address: &str) -> bool {
    if address.len() != 42 || !address.starts_with("0x") {
        return false;
    }
    
    // Check if all characters after 0x are valid hex
    address[2..].chars().all(|c| c.is_ascii_hexdigit())
}

/// Normalize Ethereum address to lowercase
pub fn normalize_ethereum_address(address: &str) -> String {
    address.to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_is_valid_ethereum_address() {
        assert!(is_valid_ethereum_address("0x1234567890123456789012345678901234567890"));
        assert!(is_valid_ethereum_address("0xabcdefABCDEF1234567890123456789012345678"));
        assert!(!is_valid_ethereum_address("0x123")); // Too short
        assert!(!is_valid_ethereum_address("1234567890123456789012345678901234567890")); // No 0x prefix
        assert!(!is_valid_ethereum_address("0x123456789012345678901234567890123456789g")); // Invalid hex
    }
    
    #[test]
    fn test_normalize_ethereum_address() {
        assert_eq!(
            normalize_ethereum_address("0xABCDEF1234567890123456789012345678901234"),
            "0xabcdef1234567890123456789012345678901234"
        );
    }
    
    #[test]
    fn test_parse_signature() {
        let sig = "0x1234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456";
        let result = parse_signature(sig);
        assert!(result.is_ok());
        
        let (r, s, v) = result.unwrap();
        assert_eq!(r.len(), 32);
        assert_eq!(s.len(), 32);
        assert_eq!(v, 0x56); // Last byte
    }
    
    #[test]
    fn test_hash_message_ethereum() {
        let message = "Hello, Ethereum!";
        let hash = hash_message_ethereum(message);
        assert_eq!(hash.len(), 32); // Keccak256 produces 32-byte hash
    }
}