use hex;
use secp256k1::Secp256k1;
use rust_blockchain::wallet::keygen::keygen;
use rust_blockchain::wallet::address::generate_base58check_address;

fn main() {
    let secp = Secp256k1::new();
    
    let (sk, pk) = keygen(secp.clone());
    let address = generate_base58check_address(&pk);
    
    println!("-------- Création d'une Paire de clé --------");
    println!("Clé privée: {}", hex::encode(&sk));
    println!("Clé publique: {}", hex::encode(&pk));
    println!("Adresse (hash160): {}", address);
}