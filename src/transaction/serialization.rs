use crate::transaction::{Tx, TxIn, TxOut, OutPoint};
use crate::script::{Witness, ScriptPubKey};
use crate::utils::{write_varint, read_varint};

/// Sérialise la transaction.
pub fn serialize(tx: &Tx, include_witness: bool) -> Vec<u8> {
    let mut data = Vec::new();
    
    data.extend_from_slice(&tx.version.to_le_bytes());
    write_varint(tx.inputs.len() as u64, &mut data);
    for txin in &tx.inputs {
        data.extend_from_slice(&txin.previous_output.txid);
        data.extend_from_slice(&txin.previous_output.vout.to_le_bytes());
        data.extend_from_slice(&txin.sequence.to_le_bytes());
    }
    
    write_varint(tx.outputs.len() as u64, &mut data);
    for txout in &tx.outputs {
        data.extend_from_slice(&txout.value.to_le_bytes());
        let spk = txout.script_pubkey.to_bytes();
        write_varint(spk.len() as u64, &mut data);
        data.extend_from_slice(&spk);
    }
    
    if include_witness {
        for txin in &tx.inputs {
            write_varint(txin.witness.items.len() as u64, &mut data);
            for item in &txin.witness.items {
                write_varint(item.len() as u64, &mut data);
                data.extend_from_slice(item);
            }
        }
    }
    
    data.extend_from_slice(&tx.lock_time.to_le_bytes());
    data
}

/// Désérialisation d'une transaction depuis un tableau d'octets.
pub fn deserialize(data: &[u8], include_witness: bool) -> Result<Tx, String> {
    let mut pos = 0;
    
    // Version (4 octets)
    if pos + 4 > data.len() {
        return Err("Pas assez d'octets pour la version".to_string());
    }
    let version = u32::from_le_bytes(data[pos..pos+4].try_into().unwrap());
    pos += 4;
    
    // Nombre d'inputs (VarInt)
    let num_inputs = read_varint(data, &mut pos)?;
    let mut inputs = Vec::new();
    for _ in 0..num_inputs {
        // txid (32 octets)
        if pos + 32 > data.len() {
            return Err("Pas assez d'octets pour un txid d'input".to_string());
        }
        let txid: [u8; 32] = data[pos..pos+32].try_into().unwrap();
        pos += 32;
        
        // vout (4 octets)
        if pos + 4 > data.len() {
            return Err("Pas assez d'octets pour un vout d'input".to_string());
        }
        let vout = u32::from_le_bytes(data[pos..pos+4].try_into().unwrap());
        pos += 4;
        
        // sequence (4 octets)
        if pos + 4 > data.len() {
            return Err("Pas assez d'octets pour une sequence d'input".to_string());
        }
        let sequence = u32::from_le_bytes(data[pos..pos+4].try_into().unwrap());
        pos += 4;
        
        // On initialise le witness à vide, il sera rempli si include_witness est true.
        inputs.push(TxIn {
            previous_output: OutPoint { txid, vout },
            sequence,
            witness: Witness { items: vec![] },
        });
    }
    
    // Nombre d'outputs (VarInt)
    let num_outputs = read_varint(data, &mut pos)?;
    let mut outputs = Vec::new();
    for _ in 0..num_outputs {
        // value (8 octets)
        if pos + 8 > data.len() {
            return Err("Pas assez d'octets pour la valeur d'une output".to_string());
        }
        let value = u64::from_le_bytes(data[pos..pos+8].try_into().unwrap());
        pos += 8;
        
        // Longueur du scriptPubKey (VarInt)
        let spk_len = read_varint(data, &mut pos)? as usize;
        if pos + spk_len > data.len() {
            return Err("Pas assez d'octets pour le scriptPubKey d'une output".to_string());
        }
        let spk_bytes = data[pos..pos+spk_len].to_vec();
        pos += spk_len;
        
        // Pour simplifier, nous reconnaissons ici quelques formats standards.
        let script_pubkey = if spk_bytes.len() >= 2 && spk_bytes[0] == 0x00 {
            // Si le deuxième octet indique la taille
            if spk_bytes[1] == 0x14 && spk_bytes.len() == 2 + 20 {
                ScriptPubKey::P2WPKH(spk_bytes[2..].to_vec())
            } else if spk_bytes[1] == 0x20 && spk_bytes.len() == 2 + 32 {
                ScriptPubKey::P2WSH(spk_bytes[2..].to_vec())
            } else {
                return Err("Format de scriptPubKey inconnu (witness)".to_string());
            }
        } else if spk_bytes.len() == 33 && spk_bytes[0] == 0x51 {
            // Pour Taproot, on attend 0x51 suivi de 32 octets
            ScriptPubKey::P2TR(spk_bytes[1..].to_vec())
        } else {
            return Err("Format de scriptPubKey inconnu".to_string());
        };
        
        outputs.push(TxOut { value, script_pubkey });
    }
    
    // Si include_witness est vrai, désérialiser la partie witness pour chaque input
    if include_witness {
        for input in inputs.iter_mut() {
            let num_items = read_varint(data, &mut pos)? as usize;
            let mut items = Vec::new();
            for _ in 0..num_items {
                let item_len = read_varint(data, &mut pos)? as usize;
                if pos + item_len > data.len() {
                    return Err("Pas assez d'octets pour un item witness".to_string());
                }
                let item = data[pos..pos+item_len].to_vec();
                pos += item_len;
                items.push(item);
            }
            input.witness = Witness { items };
        }
    }
    
    // Lock_time (4 octets)
    if pos + 4 > data.len() {
        return Err("Pas assez d'octets pour le lock_time".to_string());
    }
    let lock_time = u32::from_le_bytes(data[pos..pos+4].try_into().unwrap());
    
    Ok(Tx {
        version,
        inputs,
        outputs,
        lock_time,
    })
}
