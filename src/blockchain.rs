use serde::{Serialize, Deserialize};
use secp256k1::Secp256k1;
use blake3;

use crate::transaction::{Transaction, SigHashType};
use crate::db::make_key;


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HeadUTXO {
    pub txid: [u8; 32],
    pub vout: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UTXO {
    pub txid: [u8; 32],
    pub vout: u32,
    pub value: u64,
    pub recipient_hash: [u8; 32],
}

pub fn find_utxo(new_utxos: &[UTXO], txid: [u8; 32], vout: u32) -> Option<UTXO> {
    new_utxos
        .iter()
        .find(|utxo| utxo.txid == txid && utxo.vout == vout)
        .cloned()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub version: u32,
    pub height: u32,
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
        height: u32,
        previous_block_hash: [u8; 32],
        timestamp: u64,
        difficulty: u32,
        nonce: u32,
        transactions: Vec<Transaction>,
    ) -> Self {
        let merkle_root = Self::compute_merkle_root(&transactions);
        Block {
            version,
            height,
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

    pub fn is_header_valid(&self, difficulty: u32, reward: u64, last_hash: [u8; 32], block_height: u32) -> bool {
        // Vérifie que le bloc suit le bloc précédent
        if self.previous_block_hash != last_hash {
            println!("Bloc précédent invalide");
            return false;
        }
        // Vérifie que le bloc a la bonne hauteur
        if self.height != block_height {
            println!("Hauteur invalide");
            return false;
        }
        // Vérifie que le bloc est miné correctement
        if !Self::meets_difficulty(self.header_hash(), difficulty) {
            println!("Difficulté invalide");
            return false;
        }
        // Vérification du merkle root
        if self.merkle_root != Self::compute_merkle_root(&self.transactions) {
            println!("Merkle root invalide");
            return false;
        }
        if self.transactions.len() == 0 {
            println!("Bloc vide");
            return false;
        }
        return true;
    }
}

#[derive(Debug, Clone)]
pub struct Blockchain {
    pub secp: Secp256k1<secp256k1::All>,
    pub head: [u8; 32],
    pub mempool: Vec<Transaction>,
    pub db_utxo: sled::Db,
    pub db_block: sled::Db,
    pub test: bool,
}

impl Blockchain {
    pub fn new(secp: Secp256k1<secp256k1::All>, db_utxo_name: &str, db_block_name: &str, genesis_block: Block, test: bool) -> Result<Blockchain, Box<dyn std::error::Error>> {
        if test {
            let db_utxo = sled::Config::new()
                .temporary(true)
                .open()
                .expect("Impossible d'ouvrir la base de données UTXO");
            let db_block = sled::Config::new()
                .temporary(true)
                .open()
                .expect("Impossible d'ouvrir la base de données des blocs");
            let genesis_hash = genesis_block.header_hash();
            db_block.insert(genesis_hash, bincode::serialize(&genesis_block)?)?;
            return Ok(Blockchain {
                secp,
                head: genesis_block.header_hash(),
                mempool: vec![],
                db_utxo,
                db_block,
                test,
            });
        }
        let db_utxo = sled::open(db_utxo_name)?;
        let db_block = sled::open(db_block_name)?;
        let head = genesis_block.header_hash();
        db_block.insert(head, bincode::serialize(&genesis_block)?)?;
        Ok(Blockchain {
            secp,
            head,
            mempool: vec![],
            db_utxo,
            db_block,
            test,
        })
    }

    pub fn height(&self) -> u32 {
        let head_block = self.db_block.get(&self.head).unwrap().unwrap();
        let block: Block = bincode::deserialize(&head_block).unwrap();
        block.height
    }

    pub fn add(&mut self, block: Block) -> Result<(), Box<dyn std::error::Error>> {
        let block_hash = block.header_hash();
        let head_block = self.db_block.get(&self.head).unwrap().unwrap();
        self.db_block.insert(block_hash, bincode::serialize(&block)?)?;
        self.head = block_hash;
        Ok(())
    }

    pub fn valid_transaction(
        &self,
        tx: &Transaction,
        new_utxos: &Vec<UTXO>,
        use_utxos: &Vec<HeadUTXO>
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let mut total_input: u64 = 0;
        let mut tempo_use_utxos = use_utxos.clone();
        for (i, txin) in tx.inputs.iter().enumerate() {
            let key = make_key(txin.previous_txid, txin.previous_vout);
            // On tente d'abord de trouver l'UTXO dans la base de données.
            let utxo: UTXO = if let Some(db_value) = self.db_utxo.get(&key)? {
                // Si trouvé, on désérialise le UTXO.
                bincode::deserialize(&db_value)?
            } else if let Some(new_utxo) = find_utxo(new_utxos, txin.previous_txid, txin.previous_vout) {
                if tempo_use_utxos.contains(&HeadUTXO { txid: txin.previous_txid, vout: txin.previous_vout }) {
                    println!("Double dépense détectée");
                    return Ok(false);
                }
                new_utxo
            } else {
                // Si l'UTXO n'est trouvé ni dans la DB ni dans new_utxos, la transaction n'est pas valide.
                println!("UTXO introuvable");
                return Ok(false);
            };
    
            // On ajoute l'UTXO dans la liste des UTXO utilisés pour empêcher une double dépense dans le même bloc.
            tempo_use_utxos.push(HeadUTXO { txid: txin.previous_txid, vout: txin.previous_vout });


            // On vérifie que la signature est valide.
            let valid = tx.verify_input(&self.secp, i, txin.signature.clone(), txin.pubkey, SigHashType::All);
            if !valid {
                println!("Signature invalide");
                return Ok(false);
            }
            total_input += utxo.value;
        }

        let total_output: u64 = tx.outputs.iter().map(|output| output.value).sum();
        if total_input < total_output {
            println!("Montant insuffisant");
            return Ok(false);
        }

        println!("Transaction valide");
        Ok(true)
    }
    
    pub fn process_block(&mut self, difficulty: u32, reward: u64, block: Block) -> Result<bool, Box<dyn std::error::Error>> {
        let head_block = self.db_block.get(&self.head).unwrap().unwrap();
        let head: Block = bincode::deserialize(&head_block)?;
        if !block.is_header_valid(difficulty, reward, head.header_hash(), head.height + 1) {
            println!("Bloc header invalide");
            return Ok(false);
        }

        let mut new_utxos: Vec<UTXO> = vec![];
        let mut use_utxos: Vec<HeadUTXO> = vec![];

        let coinbase_tx = block.transactions[0].clone();

        if !coinbase_tx.is_valid_coinbase(head.height + 1, reward) {
            println!("Coinbase invalide");
            return Ok(false);
        }

        new_utxos.push(UTXO {
            txid: coinbase_tx.tx_hash(),
            vout: 0,
            value: coinbase_tx.outputs[0].value,
            recipient_hash: coinbase_tx.outputs[0].recipient_hash,
        });
        
        for tx in block.transactions.iter().skip(1) {
            if !self.valid_transaction(tx, &new_utxos, &use_utxos)? {
                println!("Transaction invalide");
                return Ok(false);
            }
            for (i, txout) in tx.outputs.iter().enumerate() {
                let key = make_key(tx.tx_hash(), i as u32);
                let utxo = UTXO {
                    txid: tx.tx_hash(),
                    vout: i as u32,
                    value: txout.value,
                    recipient_hash: txout.recipient_hash,
                };
                new_utxos.push(utxo.clone());
            }
            for (i, txin) in tx.inputs.iter().enumerate() {
                use_utxos.push(HeadUTXO {
                    txid: txin.previous_txid,
                    vout: txin.previous_vout
                });
            }
        }
        self.add(block)?;

        for utxo in new_utxos {
            let key = make_key(utxo.txid, utxo.vout);
            self.db_utxo.insert(key, bincode::serialize(&utxo)?)?;
        }
        for utxo in use_utxos {
            let key = make_key(utxo.txid, utxo.vout);
            self.db_utxo.remove(key)?;
        }

        println!("Bloc traité");
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wallet::{keygen, pubkey_to_address};
    use crate::transaction::{Transaction, TxIn, TxOut, TxInData, create_coinbase_transaction};

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
        let block1 = Block::new(1, 0, [0u8; 32], timestamp, 0, 0, vec![tx.clone()]);
        let block2 = Block::new(1, 0, [0u8; 32], timestamp, 0, 1, vec![tx]);
        let header_hash1 = block1.header_hash();
        let header_hash2 = block2.header_hash();
        assert_ne!(header_hash1, header_hash2, "Des nonce différents doivent produire des header hash différents");
    }

    #[test]
    fn test_mine_block() {
        let tx = create_dummy_transaction();
        let timestamp = 1000;
        let mut block = Block::new(1, 0, [0u8; 32], timestamp, 5, 0, vec![tx]);

        let pre_mine_hash = block.header_hash();
        assert!(!Block::meets_difficulty(pre_mine_hash, block.difficulty), "Le bloc ne doit pas être encore miné");

        block.mine();

        let mined_hash = block.header_hash();
        assert!(Block::meets_difficulty(mined_hash, block.difficulty), "Le bloc doit être miné et respecter la difficulté");
    }

    #[test]
    fn test_blockchain_new() {
        
        let secp = Secp256k1::new();
        let genesis_block = Block::new(1, 0, [0u8; 32], 0, 0, 0, vec![]);
        let mut blockchain = Blockchain::new(secp.clone(), "utxo_test", "block_test", genesis_block.clone(), true).unwrap();
        let (_sk, pk) = keygen(secp);
        let address = pubkey_to_address(&pk);

        let coinbase_tx = create_coinbase_transaction(50, address, 1);
        let block1 = Block::new(1, 1, genesis_block.header_hash(), 1, 0, 0, vec![coinbase_tx]);
        blockchain.process_block(1, 50, block1).unwrap();

    }
}

