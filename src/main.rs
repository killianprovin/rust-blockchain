use secp256k1::Secp256k1;
use rust_blockchain::wallet::keygen::keygen;
use rust_blockchain::wallet::address::generate_base58check_address;

fn main() {
    let secp = Secp256k1::new();
    
    let (_sk, pk) = keygen(secp.clone());
    let address = generate_base58check_address(&pk);
    
    println!("Adresse : {}", address);
}