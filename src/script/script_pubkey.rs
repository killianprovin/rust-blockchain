use crate::utils::hash160;
use sha2::{Sha256, Digest};

/// Différents types de scriptPubKey supportés
#[derive(Debug, Clone)]
pub enum ScriptPubKey {
    P2WPKH(Vec<u8>),         // Pay to Witness Public Key Hash
    P2WSH(Vec<u8>),          // Pay to Witness Script Hash
    P2TR(Vec<u8>),           // Pay to Taproot (Key Path ou Script Path)
}

impl ScriptPubKey {
    /// Génère le scriptPubKey sous forme de Vec<u8>
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            // P2WPKH : OP_0 <20-byte-hash>
            ScriptPubKey::P2WPKH(hash) => {
                let mut script = vec![0x00, 0x14]; // OP_0 + PUSH 20 octets
                script.extend_from_slice(hash);
                script
            }
            // P2WSH : OP_0 <32-byte-hash>
            ScriptPubKey::P2WSH(hash) => {
                let mut script = vec![0x00, 0x20]; // OP_0 + PUSH 32 octets
                script.extend_from_slice(hash);
                script
            }
            // P2TR : OP_1 <32-byte-output-key>
            ScriptPubKey::P2TR(output_key) => {
                let mut script = vec![0x51]; // OP_1
                script.extend_from_slice(output_key);
                script
            }
        }
    }

    /// Génère un scriptPubKey P2WPKH à partir d'une clé publique
    pub fn create_p2wpkh(pubkey: &[u8]) -> ScriptPubKey {
        // Calcul du Hash : RIPEMD160(SHA256(pubkey))
        let hash = hash160(&pubkey);
        ScriptPubKey::P2WPKH(hash.to_vec())
    }

    /// Génère un scriptPubKey P2WSH à partir d'un script complet
    pub fn create_p2wsh(redeem_script: &[u8]) -> ScriptPubKey {
        // Calcul du Hash : SHA256(redeem_script)
        let hash = Sha256::digest(redeem_script);
        ScriptPubKey::P2WSH(hash.to_vec())
    }

    /// Génère un scriptPubKey P2TR (Taproot) à partir de la clé de sortie (output key)
    pub fn create_p2tr(output_key: &[u8]) -> ScriptPubKey {
        ScriptPubKey::P2TR(output_key.to_vec())
    }
}