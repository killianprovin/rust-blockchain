use crate::script::ScriptPubKey;
use crate::script::Witness;

/// Référence à une sortie d'une transaction précédente
#[derive(Debug, Clone)]
pub struct OutPoint {
    pub txid: [u8; 32],
    pub vout: u32,
}

/// Entrée de transaction (TxIn) utilisant uniquement le witness
#[derive(Debug, Clone)]
pub struct TxIn {
    pub previous_output: OutPoint,
    pub sequence: u32,
    pub witness: Witness,
}

/// Sortie de transaction (TxOut)
#[derive(Debug, Clone)]
pub struct TxOut {
    pub value: u64,
    pub script_pubkey: ScriptPubKey,
}

/// Transaction utilisant uniquement Witness
#[derive(Debug, Clone)]
pub struct Tx {
    pub version: u32,
    pub inputs: Vec<TxIn>,
    pub outputs: Vec<TxOut>,
    pub lock_time: u32,
}
