use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Duration;

// Définition d'une transaction factice
#[derive(Debug, Clone)]
struct Transaction {
    id: u32,
}

fn main() {
    // La mempool est partagée entre plusieurs threads, on l'enveloppe dans Arc et Mutex.
    let mempool = Arc::new(Mutex::new(Vec::<Transaction>::new()));

    // Création d'un canal pour recevoir les transactions (de l'API ou des peers).
    let (tx, rx) = mpsc::channel::<Transaction>();

    // --- Thread API --- //
    // Ce thread simule la réception de transactions depuis une API web.
    let tx_api = tx.clone();
    let api_thread = thread::spawn(move || {
        for i in 0..5 {
            let transaction = Transaction { id: i };
            println!("[API] Nouvelle transaction reçue: {:?}", transaction);
            tx_api.send(transaction).expect("Erreur lors de l'envoi de la transaction depuis l'API");
            thread::sleep(Duration::from_millis(500));
        }
    });

    // --- Thread Peer --- //
    // Ce thread simule la réception de transactions depuis des pairs.
    let tx_peer = tx.clone();
    let peer_thread = thread::spawn(move || {
        for i in 100..105 {
            let transaction = Transaction { id: i };
            println!("[Peer] Nouvelle transaction reçue: {:?}", transaction);
            tx_peer.send(transaction).expect("Erreur lors de l'envoi de la transaction depuis un peer");
            thread::sleep(Duration::from_millis(700));
        }
    });

    // --- Thread Updater --- //
    // Ce thread reçoit les transactions depuis le canal et met à jour la mempool partagée.
    let mempool_updater = Arc::clone(&mempool);
    let updater_thread = thread::spawn(move || {
        // La boucle s'exécute tant que le canal est ouvert.
        for transaction in rx {
            {
                let mut pool = mempool_updater.lock().unwrap();
                pool.push(transaction.clone());
                println!("[Updater] Mempool mise à jour: {:?}", *pool);
            }
        }
        println!("[Updater] Fin du canal, arrêt de la mise à jour de la mempool");
    });

    // --- Thread Mining --- //
    // Ce thread simule le minage en lisant régulièrement la mempool.
    let mempool_mining = Arc::clone(&mempool);
    let mining_thread = thread::spawn(move || {
        // Ce thread s'exécute indéfiniment dans cet exemple.
        // Dans une application réelle, il faudrait prévoir un mécanisme d'arrêt.
        for _ in 0..10 {
            {
                let pool = mempool_mining.lock().unwrap();
                if !pool.is_empty() {
                    println!("[Mining] Mempool contient {} transaction(s)", pool.len());
                    // Ici, on pourrait sélectionner des transactions et tenter de miner un bloc.
                } else {
                    println!("[Mining] Mempool vide, en attente de transactions...");
                }
            }
            thread::sleep(Duration::from_secs(1));
        }
        println!("[Mining] Fin du thread de minage (simulation terminée)");
    });

    // Attente de la fin des threads API et Peer.
    api_thread.join().unwrap();
    peer_thread.join().unwrap();

    // Pour clore proprement le canal et terminer le thread updater, on droppe le dernier émetteur.
    drop(tx);
    updater_thread.join().unwrap();

    // Attendre la fin du thread de minage.
    mining_thread.join().unwrap();

    println!("Arrêt de l'application.");
}
