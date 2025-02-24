use crate::utils::{double_sha256, hash160};
use bs58;

/// Calcule le checksum (4 premiers octets du double SHA-256)
fn calculate_checksum(data: &[u8]) -> Vec<u8> {
    let hash = double_sha256(data);
    hash[0..4].to_vec()
}

/// Génère une adresse Bitcoin en Base58Check (P2PKH)
pub fn generate_base58check_address(pubkey: &[u8]) -> String {
    // Étape 1 : Hash160 de la clé publique
    let hash160 = hash160(pubkey);

    // Étape 2 : Ajout du préfixe (version)
    // Pour P2PKH (Legacy), le préfixe est 0x00
    let mut versioned_payload = vec![0x00];
    versioned_payload.extend_from_slice(&hash160);

    // Étape 3 : Calcul du checksum
    let checksum = calculate_checksum(&versioned_payload);

    // Étape 4 : Ajout du checksum
    let mut full_payload = versioned_payload;
    full_payload.extend_from_slice(&checksum);

    // Étape 5 : Encodage en Base58
    bs58::encode(full_payload).into_string()
}
