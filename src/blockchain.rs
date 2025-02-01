use serde::{Serialize, Deserialize};
use blake3;

use crate::transaction::Transaction;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub version: u32,
    pub previous_block_hash: [u8; 32],
    pub merkle_root: [u8; 32],
    pub timestamp: u64,
    pub difficulty: u32,
    pub nonce: u32,
    pub transactions: Vec<Transaction>,
}

impl Block {
    pub fn new(
        version: u32,
        previous_block_hash: [u8; 32],
        timestamp: u64,
        difficulty: u32,
        nonce: u32,
        transactions: Vec<Transaction>,
    ) -> Self {
        let merkle_root = Self::compute_merkle_root(&transactions);
        Block {
            version,
            previous_block_hash,
            merkle_root,
            timestamp,
            difficulty,
            nonce,
            transactions,
        }
    }

    pub fn compute_merkle_root(transactions: &Vec<Transaction>) -> [u8; 32] {
        let mut hashes: Vec<[u8; 32]> = transactions
            .iter()
            .map(|tx| tx.tx_hash())
            .collect();

        if hashes.is_empty() {
            return [0u8; 32];
        }

        while hashes.len() > 1 {
            if hashes.len() % 2 != 0 {
                hashes.push(*hashes.last().unwrap());
            }
            let mut new_hashes = vec![];
            for i in (0..hashes.len()).step_by(2) {
                let mut combined = Vec::with_capacity(64);
                combined.extend_from_slice(&hashes[i]);
                combined.extend_from_slice(&hashes[i + 1]);
                let new_hash = blake3::hash(&combined);
                new_hashes.push(*new_hash.as_bytes());
            }
            hashes = new_hashes;
        }
        hashes[0]
    }

    pub fn header_hash(&self) -> [u8; 32] {
        let mut data = Vec::new();
        data.extend_from_slice(&self.version.to_le_bytes());
        data.extend_from_slice(&self.previous_block_hash);
        data.extend_from_slice(&self.merkle_root);
        data.extend_from_slice(&self.timestamp.to_le_bytes());
        data.extend_from_slice(&self.difficulty.to_le_bytes());
        data.extend_from_slice(&self.nonce.to_le_bytes());
        let hash = blake3::hash(&data);
        *hash.as_bytes()
    }

    pub fn mine(&mut self) {
        while !Self::meets_difficulty(self.header_hash(), self.difficulty) {
            self.nonce = self.nonce.wrapping_add(1);
        }
    }

    fn meets_difficulty(hash: [u8; 32], difficulty: u32) -> bool {
        let mut zero_bits = difficulty;
        for byte in hash.iter() {
            if zero_bits >= 8 {
                if *byte != 0 {
                    return false;
                }
                zero_bits -= 8;
            } else if zero_bits > 0 {
                let mask = 0xFF << (8 - zero_bits);
                if *byte & mask != 0 {
                    return false;
                }
                break;
            } else {
                break;
            }
        }
        true
    }

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blockchain {
    pub blocks: Vec<Block>,
}

impl Blockchain {
    pub fn new(genesis_block: Block) -> Self {
        Blockchain {
            blocks: vec![genesis_block],
        }
    }

    pub fn add_block(&mut self, block: Block) {
        self.blocks.push(block);
    }

    pub fn latest_block(&self) -> &Block {
        self.blocks.last().expect("La blockchain ne contient aucun bloc")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::{Transaction, TxIn, TxOut, TxInData};

    fn create_dummy_transaction() -> Transaction {
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
            value: 50,
            recipient_hash: dummy_recipient_hash,
        };

        Transaction {
            version: 1,
            inputs: vec![txin],
            outputs: vec![txout],
            lock_time: 0,
        }
    }

    #[test]
    fn test_compute_merkle_root_with_transactions() {
        let tx1 = create_dummy_transaction();
        let tx2 = create_dummy_transaction();
        let transactions = vec![tx1, tx2];

        let merkle_root = Block::compute_merkle_root(&transactions);
        assert_ne!(merkle_root, [0u8; 32]);
    }

    #[test]
    fn test_compute_merkle_root_empty() {
        let transactions: Vec<Transaction> = vec![];
        let merkle_root = Block::compute_merkle_root(&transactions);
        assert_eq!(merkle_root, [0u8; 32]);
    }

    #[test]
    fn test_header_hash_changes_with_nonce() {
        let tx = create_dummy_transaction();
        let timestamp = 1000;
        let block1 = Block::new(1, [0u8; 32], timestamp, 0, 0, vec![tx.clone()]);
        let block2 = Block::new(1, [0u8; 32], timestamp, 0, 1, vec![tx]);
        let header_hash1 = block1.header_hash();
        let header_hash2 = block2.header_hash();
        assert_ne!(header_hash1, header_hash2, "Des nonce différents doivent produire des header hash différents");
    }

    #[test]
    fn test_blockchain_add_and_latest_block() {
        let tx = create_dummy_transaction();
        let timestamp = 1000;
        let genesis_block = Block::new(1, [0u8; 32], timestamp, 0, 0, vec![tx.clone()]);
        let mut blockchain = Blockchain::new(genesis_block.clone());
        assert_eq!(blockchain.latest_block().header_hash(), genesis_block.header_hash());

        let new_block = Block::new(1, genesis_block.header_hash(), timestamp + 1, 0, 0, vec![tx]);
        blockchain.add_block(new_block.clone());

        assert_eq!(blockchain.latest_block().header_hash(), new_block.header_hash());
        assert_eq!(blockchain.blocks.len(), 2);
    }

    #[test]
    fn test_mine_block() {
        let tx = create_dummy_transaction();
        let timestamp = 1000;
        let mut block = Block::new(1, [0u8; 32], timestamp, 5, 0, vec![tx]);

        let pre_mine_hash = block.header_hash();
        assert!(!Block::meets_difficulty(pre_mine_hash, block.difficulty), "Le bloc ne doit pas être encore miné");

        block.mine();

        let mined_hash = block.header_hash();
        assert!(Block::meets_difficulty(mined_hash, block.difficulty), "Le bloc doit être miné et respecter la difficulté");
    }
}
