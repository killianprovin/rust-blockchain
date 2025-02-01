use sled;

use rust_blockchain::transaction::{create_coinbase_transaction};
use rust_blockchain::blockchain::Block;
use rust_blockchain::db::{store_transaction, store_block, get_latest_block};


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
    let mut block1 = Block::new(2, genesis_block.header_hash(), 1002, 16, 0, vec![tx1.clone(), tx2.clone()]);
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
