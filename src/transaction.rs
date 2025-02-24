use secp256k1::{Secp256k1, Message, Keypair, SecretKey, XOnlyPublicKey};
use serde::{Serialize, Deserialize};
use secp256k1::schnorr::Signature;
use crate::wallet::double_sha256;

// Séparartion de la SchnorrSignature en deux parties r et s pour la sérialisation
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinbaseData {
    pub block_height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TxInData {
    Standard,
    Coinbase(CoinbaseData),
}

// Structure entrée de transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxIn {
    pub previous_txid: [u8; 32],
    pub previous_vout: u32,
    pub pubkey: [u8; 32],
    pub signature: SchnorrSignature,
    pub tx_in_data: Option<TxInData>,
}


// Structure sortie de transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxOut {
    pub value: u64,
    pub recipient_hash: [u8; 32],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub version: u32,
    pub inputs: Vec<TxIn>,
    pub outputs: Vec<TxOut>,
    pub lock_time: u32,
}

impl Transaction {
    pub fn tx_hash(&self) -> [u8; 32] {
        let mut data = Vec::new();

        data.extend_from_slice(&self.version.to_le_bytes());

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
        for output in &self.outputs {
            data.extend_from_slice(&output.value.to_le_bytes());
            data.extend_from_slice(&output.recipient_hash);
        }

        data.extend_from_slice(&self.lock_time.to_le_bytes());

        double_sha256(&data)
    }

    // Signe une entrée de la transaction
    pub fn sign_input(
        &mut self,
        secp: &Secp256k1<secp256k1::All>,
        input_index: usize,
        secret_bytes: [u8; 32],
    ) {
        let secret_key = SecretKey::from_slice(&secret_bytes)
            .expect("Clé privée invalide");
        let hash = self.tx_hash();
        let message = Message::from_digest(hash);
        let keypair = Keypair::from_secret_key(secp, &secret_key);
        let signature = secp.sign_schnorr(message.as_ref(), &keypair);
        self.inputs[input_index].signature = signature.to_byte_array().into();
    }
    
    // Vérifie la signature d'une entrée donnée
    pub fn verify_input(
        &self,
        secp: &Secp256k1<secp256k1::All>,
        signature: SchnorrSignature,
        pubkey: [u8; 32],
    ) -> bool {
        let hash = self.tx_hash();
        let message = Message::from_digest(hash);
        let xonly_pubkey = XOnlyPublicKey::from_slice(&pubkey)
            .expect("Clé publique invalide");
        let sig = Signature::from_byte_array(signature.into());
        secp.verify_schnorr(&sig, message.as_ref(), &xonly_pubkey).is_ok()
    }


    // Vérifie que la coinbase est valide (à modifier avec les fraits de tranasaction)
    pub fn is_valid_coinbase(&self, block_height: u32, reward: u64) -> bool {
        if self.inputs.len() != 1 {
            return false;
        }
        if self.outputs.len() != 1 {
            return false;
        }
        if self.outputs[0].value != reward {
            return false;
        }
        if self.inputs[0].previous_txid != [0u8; 32] {
            return false;
        }
        if self.inputs[0].previous_vout != u32::MAX {
            return false;
        }
        if self.inputs[0].pubkey != [0u8; 32] {
            return false;
        }
        if self.inputs[0].signature.r != [0u8; 32] || self.inputs[0].signature.s != [0u8; 32] {
            return false;
        }
        if let Some(TxInData::Coinbase(coinbase_data)) = &self.inputs[0].tx_in_data {
            if coinbase_data.block_height != block_height {
                return false;
            }
        } else {
            return false;
        }
        true
    }
}

// Crée une transaction coinbase
pub fn create_coinbase_transaction(reward: u64, miner_address: [u8; 32], block_height: u32) -> Transaction {
    let coinbase_input = TxIn {
        previous_txid: [0u8; 32],
        previous_vout: u32::MAX,
        pubkey: [0u8; 32],
        signature: SchnorrSignature { r: [0u8; 32], s: [0u8; 32] },
        tx_in_data: Some(TxInData::Coinbase(CoinbaseData { block_height })),
    };

    let coinbase_output = TxOut {
        value: reward,
        recipient_hash: miner_address,
    };

    Transaction {
        version: 1,
        inputs: vec![coinbase_input],
        outputs: vec![coinbase_output],
        lock_time: 0,
    }
}