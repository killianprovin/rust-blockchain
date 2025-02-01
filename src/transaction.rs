use secp256k1::{Secp256k1, Message, Keypair, SecretKey, XOnlyPublicKey};
use serde::{Serialize, Deserialize};
use secp256k1::schnorr::Signature;
use blake3;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchnorrSignature {
    pub r: [u8; 32],
    pub s: [u8; 32],
}

// Conversion depuis un tableau de 64 octets vers SchnorrSignature
impl From<[u8; 64]> for SchnorrSignature {
    fn from(sig: [u8; 64]) -> Self {
        let mut r = [0u8; 32];
        let mut s = [0u8; 32];
        r.copy_from_slice(&sig[..32]);
        s.copy_from_slice(&sig[32..]);
        SchnorrSignature { r, s }
    }
}

// Conversion inverse, pour obtenir un [u8; 64] à partir d'une SchnorrSignature
impl From<SchnorrSignature> for [u8; 64] {
    fn from(sig: SchnorrSignature) -> [u8; 64] {
        let mut res = [0u8; 64];
        res[..32].copy_from_slice(&sig.r);
        res[32..].copy_from_slice(&sig.s);
        res
    }
}

/// Modes de signature
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SigHashType {
    All,
    None,
    Single,
    AllAnyoneCanPay,
    NoneAnyoneCanPay,
    SingleAnyoneCanPay,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinbaseData {
    pub block_height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TxInData {
    Standard,
    Coinbase(CoinbaseData),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxIn {
    pub previous_txid: [u8; 32],
    pub previous_vout: u32,
    pub pubkey: [u8; 32],
    pub signature: SchnorrSignature,
    pub tx_in_data: Option<TxInData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxOut {
    pub value: u64,
    pub recipient_hash: [u8; 32],
}

// Une transaction composée de plusieurs entrées et sorties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub version: u32,
    pub inputs: Vec<TxIn>,
    pub outputs: Vec<TxOut>,
    pub lock_time: u32,
}

impl Transaction {
    // Construit la pré-image de la transaction en fonction du mode de signature et d'un éventuel indice d'input
    fn sighash_preimage(&self, sighash: SigHashType, input_index: Option<usize>) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&self.version.to_le_bytes());
        match sighash {
            SigHashType::All | SigHashType::None | SigHashType::Single => {
                for input in &self.inputs {
                    data.extend_from_slice(&input.previous_txid);
                    data.extend_from_slice(&input.previous_vout.to_le_bytes());
                    data.extend_from_slice(&input.pubkey);
                    match &input.tx_in_data {
                        Some(TxInData::Standard) => {},
                        Some(TxInData::Coinbase(coinbasedata)) => {
                            data.extend_from_slice(&coinbasedata.block_height.to_le_bytes());
                        },
                        None => panic!("Données d'entrée manquantes"),
                    }
                }
            },
            SigHashType::AllAnyoneCanPay | SigHashType::NoneAnyoneCanPay | SigHashType::SingleAnyoneCanPay => {
                if let Some(idx) = input_index {
                    if idx < self.inputs.len() {
                        let input = &self.inputs[idx];
                        data.extend_from_slice(&input.previous_txid);
                        data.extend_from_slice(&input.previous_vout.to_le_bytes());
                        data.extend_from_slice(&input.pubkey);
                        match &input.tx_in_data {
                            Some(TxInData::Standard) => {},
                            Some(TxInData::Coinbase(coinbasedata)) => {
                                data.extend_from_slice(&coinbasedata.block_height.to_le_bytes());
                            },
                            None => panic!("Données d'entrée manquantes"),
                        }
                    } else {
                        panic!("Indice d'entrée invalide pour le mode ANYONECANPAY");
                    }
                } else {
                    panic!("Indice d'entrée requis pour le mode ANYONECANPAY");
                }
            }
        }
        match sighash {
            SigHashType::All | SigHashType::AllAnyoneCanPay => {
                for output in &self.outputs {
                    data.extend_from_slice(&output.value.to_le_bytes());
                    data.extend_from_slice(&output.recipient_hash);
                }
            },
            SigHashType::None | SigHashType::NoneAnyoneCanPay => {
                // rien
            },
            SigHashType::Single | SigHashType::SingleAnyoneCanPay => {
                if let Some(idx) = input_index {
                    if idx < self.outputs.len() {
                        let output = &self.outputs[idx];
                        data.extend_from_slice(&output.value.to_le_bytes());
                        data.extend_from_slice(&output.recipient_hash);
                    } else {
                        panic!("Indice de sortie invalide pour SIGHASH_SINGLE");
                    }
                } else {
                    panic!("Indice d'entrée requis pour SIGHASH_SINGLE");
                }
            }
        }
        data.extend_from_slice(&self.lock_time.to_le_bytes());
        data
    }

    // Calcule le hash de la transaction en mode SIGHASH_ALL
    pub fn tx_hash(&self) -> [u8; 32] {
        let data = self.sighash_preimage(SigHashType::All, None);
        let hash = blake3::hash(&data);
        *hash.as_bytes()
    }

    // Signe une entrée de la transaction
    pub fn sign_input(
        &mut self,
        secp: &Secp256k1<secp256k1::All>,
        input_index: usize,
        secret_bytes: [u8; 32],
        sighash: SigHashType,
    ) {
        let secret_key = SecretKey::from_slice(&secret_bytes)
            .expect("Clé privée invalide");
        let preimage = self.sighash_preimage(sighash, Some(input_index));
        let hash = blake3::hash(&preimage);
        let message = Message::from_digest(*hash.as_bytes());
        let keypair = Keypair::from_secret_key(secp, &secret_key);
        let signature = secp.sign_schnorr(message.as_ref(), &keypair);
        self.inputs[input_index].signature = signature.to_byte_array().into();
    }
    
    // Vérifie la signature d'une entrée donnée
    pub fn verify_input(
        &self,
        secp: &Secp256k1<secp256k1::All>,
        input_index: usize,
        signature: [u8; 64],
        pubkey_bytes: [u8; 32],
        sighash: SigHashType,
    ) -> bool {
        let preimage = self.sighash_preimage(sighash, Some(input_index));
        let hash = blake3::hash(&preimage);
        let message = Message::from_digest(*hash.as_bytes());
        let xonly_pubkey = XOnlyPublicKey::from_slice(&pubkey_bytes)
            .expect("Clé publique invalide");
        let sig = Signature::from_byte_array(signature);
        secp.verify_schnorr(&sig, message.as_ref(), &xonly_pubkey).is_ok()
    }
}

pub fn create_coinbase_transaction(reward: u64, miner_address: [u8; 32], block_height: u32) -> Transaction {
    let coinbase_input = TxIn {
        previous_txid: [0u8; 32],
        previous_vout: u32::MAX,
        pubkey: [0u8; 32],
        signature: SchnorrSignature { r: [0u8; 32], s: [0u8; 32] },
        tx_in_data: Some(TxInData::Coinbase(CoinbaseData { block_height })),
    };

    // Le ou les outputs indiquent le paiement de la récompense.
    let coinbase_output = TxOut {
        value: reward,
        recipient_hash: miner_address, // L'adresse (recipient_hash) du mineur
    };

    Transaction {
        version: 1,
        inputs: vec![coinbase_input],
        outputs: vec![coinbase_output],
        lock_time: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use secp256k1::{Secp256k1, SecretKey, PublicKey};

    #[test]
    fn test_tx_hash_non_zero() {
        let dummy_txid = [1u8; 32];
        let dummy_vout = 0;
        let dummy_pubkey = [10u8; 32];
        let dummy_signature = [0u8; 64];
        let txin = TxIn {
            previous_txid: dummy_txid,
            previous_vout: dummy_vout,
            pubkey: dummy_pubkey,
            signature: dummy_signature.into(),
            tx_in_data: Some(TxInData::Standard),
        };

        let dummy_recipient_hash = [2u8; 32];
        let txout = TxOut {
            value: 100,
            recipient_hash: dummy_recipient_hash,
        };

        let tx = Transaction {
            version: 1,
            inputs: vec![txin],
            outputs: vec![txout],
            lock_time: 0,
        };

        let hash = tx.tx_hash();
        assert_ne!(hash, [0u8; 32], "Le hash de la transaction ne doit pas être nul");
    }

    #[test]
    fn test_sign_and_verify_input() {
        let secp = Secp256k1::new();
        let secret_key = SecretKey::from_slice(&[42u8; 32]).expect("32 octets valides");
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        let (xonly_pubkey, _) = PublicKey::x_only_public_key(&public_key);
        let pubkey_bytes: [u8; 32] = xonly_pubkey.serialize();

        let dummy_txid = [1u8; 32];
        let dummy_vout = 0;
        let txin = TxIn {
            previous_txid: dummy_txid,
            previous_vout: dummy_vout,
            pubkey: pubkey_bytes,
            signature: [0u8; 64].into(),
            tx_in_data: Some(TxInData::Standard),
        };

        let dummy_recipient_hash = [2u8; 32];
        let txout = TxOut {
            value: 50,
            recipient_hash: dummy_recipient_hash,
        };

        let mut tx = Transaction {
            version: 1,
            inputs: vec![txin],
            outputs: vec![txout],
            lock_time: 0,
        };

        tx.sign_input(&secp, 0, secret_key.secret_bytes(), SigHashType::All);

        let sig = &tx.inputs[0].signature;
        let valid = tx.verify_input(&secp, 0, sig.clone().into(), pubkey_bytes, SigHashType::All);
        assert!(valid, "La signature doit être valide avec la bonne clé publique");

        let other_secret = SecretKey::from_slice(&[43u8; 32]).expect("32 octets");
        let other_public = PublicKey::from_secret_key(&secp, &other_secret);
        let (other_xonly, _) = PublicKey::x_only_public_key(&other_public);
        let other_pubkey_bytes: [u8; 32] = other_xonly.serialize();
        let valid_other = tx.verify_input(&secp, 0, sig.clone().into(), other_pubkey_bytes, SigHashType::All);
        assert!(!valid_other, "La signature ne doit pas être valide avec une mauvaise clé publique");
    }

    #[test]
    #[should_panic(expected = "Indice d'entrée requis pour SIGHASH_SINGLE")]
    fn test_sighash_single_without_input_index() {
        let dummy_txid = [1u8; 32];
        let dummy_vout = 0;
        let dummy_pubkey = [10u8; 32];
        let dummy_signature = [0u8; 64];
        let txin = TxIn {
            previous_txid: dummy_txid,
            previous_vout: dummy_vout,
            pubkey: dummy_pubkey,
            signature: dummy_signature.into(),
            tx_in_data: Some(TxInData::Standard),
        };

        let dummy_recipient_hash = [2u8; 32];
        let txout = TxOut {
            value: 100,
            recipient_hash: dummy_recipient_hash,
        };

        let tx = Transaction {
            version: 1,
            inputs: vec![txin],
            outputs: vec![txout],
            lock_time: 0,
        };

        let _ = tx.sighash_preimage(SigHashType::Single, None);
    }

    #[test]
    fn test_coinbase_transaction() {
        let miner_address = [42u8; 32];
        let reward = 50;
        let tx = create_coinbase_transaction(reward, miner_address, 42);
        assert_eq!(tx.inputs.len(), 1);
        assert_eq!(tx.outputs.len(), 1);
        assert_eq!(tx.inputs[0].previous_txid, [0u8; 32]);
        assert_eq!(tx.inputs[0].previous_vout, u32::MAX);
        assert_eq!(tx.outputs[0].value, reward);
        assert_eq!(tx.outputs[0].recipient_hash, miner_address);
    }
}
