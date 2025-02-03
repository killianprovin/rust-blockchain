use secp256k1::Secp256k1;
use serde::{Serialize, Deserialize};

use rust_blockchain::wallet::{keygen, pubkey_to_address};
use rust_blockchain::transaction::{Transaction, TxIn, TxOut, TxInData, SchnorrSignature, SigHashType, create_coinbase_transaction};
use rust_blockchain::blockchain::{Block, Blockchain, UTXO};
use rust_blockchain::db::from_key;


fn liste_utxo(blockchain: &Blockchain) {
    println!("Liste des UTXO:");
    // Itérer sur la base de données db_utxo.
    for result in blockchain.db_utxo.iter() {
        // Chaque élément est un Result<(IVec, IVec), sled::Error>.
        let (key, value) = result.expect("Erreur lors de l'itération");
        let (txid, vout) = from_key(key.to_vec());

        let utxo: UTXO = bincode::deserialize(&value).expect("Erreur lors de la désérialisation");

        println!("txid: {:?}, vout: {:?}", txid, vout);
        println!("  - montant: {}", utxo.value);
        println!("  - destinataire: {:?}", utxo.recipient_hash);
    }
}

fn main() {
    let secp = Secp256k1::new();
    let genesis_block = Block::new(1, 0, [0u8; 32], 0, 0, 0, vec![]);
    let mut blockchain = Blockchain::new(secp.clone(), "db_utxo", "db_block", genesis_block.clone(), true)
        .expect("Erreur lors de la création de la blockchain");
    let (sk, pk) = keygen(secp.clone());
    let (sk2, pk2) = keygen(secp.clone());
    let address = pubkey_to_address(&pk);
    let address2 = pubkey_to_address(&pk2);

    let coinbase_tx1 = create_coinbase_transaction(50, address, 1);
    let mut block1 = Block::new(1, 1, genesis_block.header_hash(), 1, 1, 0, vec![coinbase_tx1]);
    block1.mine();
    println!("Block1: {:?}", block1.header_hash());
    blockchain.process_block(1, 50, block1.clone()).unwrap();
    println!("height: {}", blockchain.height());
    liste_utxo(&blockchain);

    let coinbase_tx2 = create_coinbase_transaction(50, address, 2);
    let mut block2 = Block::new(1, 2, block1.header_hash(), 2, 2, 0, vec![coinbase_tx2.clone()]);
    block2.mine();

    println!("Block2: {:?}", block2.header_hash());
    blockchain.process_block(1, 50, block2.clone()).unwrap();
    println!("height: {}", blockchain.height());
    liste_utxo(&blockchain);

    //teste transaction depuis coinbase
    let coinbase_tx3 = create_coinbase_transaction(50, address, 3);
    let txin = TxIn {
        previous_txid: coinbase_tx2.tx_hash(),
        previous_vout: 0,
        pubkey: pk,
        tx_in_data: Some(TxInData::Standard),
        signature: SchnorrSignature { r: [0u8; 32], s: [0u8; 32] },
    };

    let txout1 = TxOut {
        value: 25,
        recipient_hash: address,
    };

    let txout2 = TxOut {
        value: 25,
        recipient_hash: address2,
    };

    let mut tx = Transaction {
        version: 1,
        inputs: vec![txin],
        outputs: vec![txout1, txout2],
        lock_time: 0,
    };

    tx.sign_input(&secp, 0, sk, SigHashType::All);

    let mut block3 = Block::new(1, 3, block2.header_hash(), 3, 3, 0, vec![coinbase_tx3, tx]);
    block3.mine();
    println!("Block3: {:?}", block3.header_hash());
    blockchain.process_block(1, 50, block3.clone()).unwrap();
    println!("height: {}", blockchain.height());
    liste_utxo(&blockchain);
}

