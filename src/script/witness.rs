/// Structure Witness : une pile d'éléments (chaque élément est un Vec<u8>)
#[derive(Debug, Clone)]
pub struct Witness {
    pub items: Vec<Vec<u8>>,
}

/// Pour P2WPKH, le witness contient généralement deux éléments :
/// 1. La signature (Schnorr ou ECDSA)
/// 2. La clé publique compressée
pub fn create_witness_p2wpkh(signature: &[u8], pubkey: &[u8]) -> Witness {
    Witness {
        items: vec![
            signature.to_vec(),
            pubkey.to_vec(),
        ],
    }
}

/// Pour P2WSH, le witness sert à fournir tous les éléments nécessaires pour exécuter
/// le script complet (redeem script). Par exemple, pour un multisig 2-of-2, le witness
/// contient les signatures suivies du redeem script
/// (voir pour ajouter CHECKMULTISIG)
pub fn create_witness_p2wsh(signatures: Vec<&[u8]>, redeem_script: &[u8]) -> Witness {
    let mut items: Vec<Vec<u8>> = Vec::new();
    for sig in signatures {
        items.push(sig.to_vec());
    }
    items.push(redeem_script.to_vec());
    Witness { items }
}