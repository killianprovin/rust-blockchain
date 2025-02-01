use sled;
use bincode;
use std::error::Error;
use crate::transaction::Transaction;
use crate::blockchain::Block;

pub fn store_transaction(db: &sled::Db, key: [u8; 32], tx: &Transaction) -> Result<(), Box<dyn Error>> {
    let encoded: Vec<u8> = bincode::serialize(tx)?;
    db.insert(key, encoded)?;
    db.flush()?;
    Ok(())
}

pub fn load_transaction(db: &sled::Db, key: [u8; 32]) -> Result<Transaction, Box<dyn Error>> {
    let data = db.get(key)?.ok_or("Clé non trouvée")?;
    let tx: Transaction = bincode::deserialize(&data)?;
    Ok(tx)
}

pub fn store_block(db: &sled::Db, block: &Block) -> Result<(), Box<dyn Error>> {
    let key = block.header_hash();
    let serialized_block = bincode::serialize(block)?;
    db.insert(key, serialized_block)?;
    // Met à jour le dernier bloc en stockant le hash du bloc sous la clé "HEAD"
    db.insert("HEAD", key.as_ref())?;
    db.flush()?;
    Ok(())
}

pub fn get_latest_block(db: &sled::Db) -> Result<Option<Block>, Box<dyn Error>> {
    if let Some(head_bytes) = db.get("HEAD")? {
        let head_hash: [u8; 32] = head_bytes.as_ref().try_into().expect("Taille invalide");
        if let Some(serialized_block) = db.get(head_hash)? {
            let block: Block = bincode::deserialize(&serialized_block)?;
            return Ok(Some(block));
        }
    }
    Ok(None)
}
