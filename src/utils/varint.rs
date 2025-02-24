/// Sérialisation d'un entier en format VarInt
pub fn write_varint(n: u64, buffer: &mut Vec<u8>) {
    if n < 0xFD {
        buffer.push(n as u8);
    } else if n <= 0xFFFF {
        buffer.push(0xFD);
        buffer.extend_from_slice(&(n as u16).to_le_bytes());
    } else if n <= 0xFFFFFFFF {
        buffer.push(0xFE);
        buffer.extend_from_slice(&(n as u32).to_le_bytes());
    } else {
        buffer.push(0xFF);
        buffer.extend_from_slice(&(n as u64).to_le_bytes());
    }
}

/// Désérialisation d'un VarInt depuis `data`
/// Met à jour `pos` pour pointer après le VarInt lu.
/// Retourne un `Result<u64, String>` pour gérer les erreurs de lecture.
pub fn read_varint(data: &[u8], pos: &mut usize) -> Result<u64, String> {
    if *pos >= data.len() {
        return Err("Pas assez d'octets pour lire le préfixe VarInt".to_string());
    }
    let prefix = data[*pos];
    *pos += 1;

    match prefix {
        0xFD => {
            if *pos + 2 > data.len() {
                return Err("Pas assez d'octets pour lire un VarInt (u16)".to_string());
            }
            let buf: [u8; 2] = data[*pos..*pos + 2].try_into().unwrap();
            *pos += 2;
            Ok(u16::from_le_bytes(buf) as u64)
        }
        0xFE => {
            if *pos + 4 > data.len() {
                return Err("Pas assez d'octets pour lire un VarInt (u32)".to_string());
            }
            let buf: [u8; 4] = data[*pos..*pos + 4].try_into().unwrap();
            *pos += 4;
            Ok(u32::from_le_bytes(buf) as u64)
        }
        0xFF => {
            if *pos + 8 > data.len() {
                return Err("Pas assez d'octets pour lire un VarInt (u64)".to_string());
            }
            let buf: [u8; 8] = data[*pos..*pos + 8].try_into().unwrap();
            *pos += 8;
            Ok(u64::from_le_bytes(buf))
        }
        _ => Ok(prefix as u64),
    }
}
