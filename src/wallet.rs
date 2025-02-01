use secp256k1::rand::rngs::OsRng;
use secp256k1::{Secp256k1, PublicKey, Keypair};
use blake3;

/// Génère une paire de clés (clé privée et clé publique) pour le wallet
/// Retourne la clé secrète sous forme de [u8; 32] et la clé publique (x-only) sous forme de [u8; 32].
pub fn keygen(secp: Secp256k1<secp256k1::All>) -> ([u8; 32], [u8; 32]) {
    let mut rng = OsRng;
    let keypair = Keypair::new(&secp, &mut rng);
    let secret_key = keypair.secret_key();
    let (xonly_pubkey, _) = PublicKey::x_only_public_key(&keypair.public_key());
    (secret_key.secret_bytes(), xonly_pubkey.serialize())
}

/// Effectue un double hachage avec Blake3
fn double_blake3(data: &[u8]) -> [u8; 32] {
    let h1 = blake3::hash(data);
    let h2 = blake3::hash(h1.as_bytes());
    *h2.as_bytes()
}

/// Convertit une clé publique (représentée sous forme de [u8; 32]) en "adresse" (recipient_hash)
pub fn pubkey_to_address(pubkey: &[u8; 32]) -> [u8; 32] {
    double_blake3(pubkey)
}

#[cfg(test)]
mod tests {
    use super::*;
    use secp256k1::Secp256k1;

    #[test]
    fn test_keygen_and_pubkey_to_address() {
        let secp = Secp256k1::new();
        let (_sk, pk) = keygen(secp);
        let address = pubkey_to_address(&pk);
        assert_ne!(address, [0u8; 32]);
    }

    #[test]
    fn test_double_blake3() {
        let data = b"test data";
        let hash = double_blake3(data);
        assert_ne!(hash, [0u8; 32]);
    }
}
