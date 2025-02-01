use rust_blockchain::blockchain::{Blockchain, Block};
use rust_blockchain::transaction::{Transaction, TxIn, TxOut};
use serde_json;

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

fn main() {
    let tx = create_dummy_transaction();

    let timestamp = 1000;
    let genesis_block = Block::new(1, [0u8; 32], timestamp, 0, 0, vec![tx]);

    let blockchain = Blockchain::new(genesis_block);

    let serialized = serde_json::to_string_pretty(&blockchain)
        .expect("Erreur lors de la sérialisation de la blockchain");
    println!("Blockchain sérialisée :\n{}", serialized);

    let deserialized: Blockchain = serde_json::from_str(&serialized)
        .expect("Erreur lors de la désérialisation de la blockchain");

    let original_hash = blockchain.latest_block().header_hash();
    let deserialized_hash = deserialized.latest_block().header_hash();
    assert_eq!(original_hash, deserialized_hash, "Les blockchains doivent être identiques après désérialisation");
}
