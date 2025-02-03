pub fn make_key(txid: [u8; 32], vout: u32) -> Vec<u8> {
    let mut key = Vec::with_capacity(32 + 4);
    key.extend_from_slice(&txid);
    key.extend_from_slice(&vout.to_le_bytes());
    key
}

pub fn from_key(key: Vec<u8>) -> ([u8; 32], u32) {
    let txid: [u8; 32] = key[0..32].try_into().expect("Longueur invalide pour txid");
    let vout = u32::from_le_bytes(key[32..36].try_into().expect("Longueur invalide pour vout"));
    (txid, vout)
}