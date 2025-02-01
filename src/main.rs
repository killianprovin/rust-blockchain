use sled;
use bincode;

use rust_blockchain::transaction::{Transaction, create_coinbase_transaction};
use rust_blockchain::blockchain::Block;

fn store_transaction(db: &sled::Db, key: [u8; 32], tx: &Transaction) -> Result<(), Box<dyn std::error::Error>> {
    // Sérialise la transaction en format binaire
    let encoded: Vec<u8> = bincode::serialize(tx)?;
    // Stocke la valeur dans la base de données avec la clé fournie
    db.insert(key, encoded)?;
    db.flush()?; // pour s'assurer que les données sont écrites sur disque
    Ok(())
}

fn load_transaction(db: &sled::Db, key: [u8; 32]) -> Result<Transaction, Box<dyn std::error::Error>> {
    // Récupère les données associées à la clé
    let data = db.get(key)?.ok_or("Clé non trouvée")?;
    // Désérialise les données en une Transaction
    let tx: Transaction = bincode::deserialize(&data)?;
    Ok(tx)
}

fn store_block(db: &sled::Db, block: &Block) -> Result<(), Box<dyn std::error::Error>> {
    let key = block.header_hash();
    let serialized_block = bincode::serialize(block)?;
    db.insert(key, serialized_block)?;
    // Met à jour le dernier bloc (HEAD)
    db.insert("HEAD", key.as_ref())?;
    db.flush()?;
    Ok(())
}

fn get_latest_block(db: &sled::Db) -> Result<Option<Block>, Box<dyn std::error::Error>> {
    if let Some(head_bytes) = db.get("HEAD")? {
        let head_hash: [u8; 32] = head_bytes.as_ref().try_into().expect("Taille invalide");
        if let Some(serialized_block) = db.get(head_hash)? {
            let block: Block = bincode::deserialize(&serialized_block)?;
            return Ok(Some(block));
        }
    }
    Ok(None)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dbutxo = sled::open("db_UTXO")?;

    let tx1 = create_coinbase_transaction(100, [42u8; 32], 10);
    let tx2 = create_coinbase_transaction(100, [42u8; 32], 11);

    store_transaction(&dbutxo, tx1.tx_hash(), &tx1)?;
    store_transaction(&dbutxo, tx2.tx_hash(), &tx2)?;

    println!("Liste UTXO:");
    for item in dbutxo.iter() {
        let (key, _value) = item?;
        println!("ID: {}", hex::encode(&key));
        dbutxo.remove(key)?;
    }

    let dbblockchain = sled::open("db_Blockchain")?;

    let mut genesis_block = Block::new(1, [0u8; 32], 1000, 16, 0, vec![tx1.clone(), tx2.clone()]);
    genesis_block.mine();
    let mut block1 = Block::new(2, genesis_block.header_hash(), 1001, 16, 0, vec![tx1.clone(), tx2.clone()]);
    block1.mine();

    store_block(&dbblockchain, &genesis_block)?;
    store_block(&dbblockchain, &block1)?;

    let latest_block = get_latest_block(&dbblockchain)?;
    if let Some(block) = latest_block {
        println!("Dernier bloc: {}", hex::encode(block.header_hash()));
    }

    println!("Liste Block:");
    for item in dbblockchain.iter() {
        let (key, _value) = item?;
        println!("ID: {}", hex::encode(&key));
        dbblockchain.remove(key)?;
    }


    Ok(())
}
