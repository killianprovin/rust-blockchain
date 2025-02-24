use secp256k1::rand::rngs::OsRng;
use secp256k1::{Secp256k1, PublicKey, Keypair};

/// Génère une paire de clés (clé privée et clé publique) pour le wallet
/// Retourne la clé secrète sous forme de [u8; 32] et la clé publique (x-only) sous forme de [u8; 32].
pub fn keygen(secp: Secp256k1<secp256k1::All>) -> ([u8; 32], [u8; 32]) {
    let mut rng = OsRng;
    let keypair = Keypair::new(&secp, &mut rng);
    let secret_key = keypair.secret_key();
    let (xonly_pubkey, _) = PublicKey::x_only_public_key(&keypair.public_key());
    (secret_key.secret_bytes(), xonly_pubkey.serialize())
}