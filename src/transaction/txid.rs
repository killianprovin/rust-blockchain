use crate::transaction::Tx;
use crate::transaction::serialization::serialize;
use crate::utils::double_sha256;

/// Calcule le txid (sans witness)
pub fn txid(tx: &Tx) -> [u8; 32] {
    let serialized = serialize(tx, false);
    double_sha256(&serialized)
}

/// Calcule le wtxid (avec witness)
pub fn wtxid(tx: &Tx) -> [u8; 32] {
    let serialized = serialize(tx, true);
    double_sha256(&serialized)
}
