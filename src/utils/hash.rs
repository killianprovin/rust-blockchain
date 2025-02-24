use sha2::{Sha256, Digest};
use ripemd::{Ripemd160};

/// Effectue un double hachage avec SHA-256
pub fn double_sha256(data: &[u8]) -> [u8; 32] {
    let hash1 = Sha256::digest(data);
    let hash2 = Sha256::digest(&hash1);
    let mut result = [0u8; 32];
    result.copy_from_slice(&hash2);
    result
}

/// Effectue un hash RIPEMD160(SHA256(pubkey))
pub fn hash160(pubkey: &[u8]) -> Vec<u8> {
    let sha256 = Sha256::digest(pubkey);
    let ripemd160 = Ripemd160::digest(&sha256);
    ripemd160.to_vec()
}
