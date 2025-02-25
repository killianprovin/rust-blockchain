Ce projet implémente une blockchain de type Bitcoin en Rust

Modèle UTXO

Preuve de Travail (POW) avec Blake3 pour le minage de blocs

Signatures Schnorr

gestion future des communications P2P, vérification complète des transactions (frais, UTXO, scripts de dépense), et intégration de Zero-Knowledge Proofs pour un layer2 ZkRollUP.


# Wallet
## Keygen
Génération des paires de clé sur la courbe élyptique Secp256k1
## Adresse
J'utilise la base58 comme ça pas de I et l il me semble, y'a aussi une check somme pour s'assurer qu'on ai pas fait d'erreurs. 
J'utilise hash160 (qui est dans utils) qui fait SHA256 puis Ripemd160.

# Utils
y'a double_sha256 que j'utilise pour les transaction et hash160 pour les adress (20 bits)

# Script 
## Script PubKey
poru l'instant on gere 3 type :
 + P2WPKH (transaction classique)
 + P2WSH (pour le multisig)
# Witness
on gère les meme type que Script PubKey
 + P2WPKH : signature + pubkey
 + P2WSH : signatures + script


# Transaction
## Tx
### OutPoint
- txid : l'id de la transaction précédente 
- vout : et l'index du out corespondant
### TxIn
 - previous_output : OutPoint
 - sequence : pour transaction retardées (absolu ou relatif)
 - witness : CF avant la preuve qui corespond au script pour dépenser
### TxOut
 - value : la valeur envoyé
 - script_pubkey : CF avant, le script pour dépenser
### Tx
version
 - inputs : CF avant
 - outputs : CF avant
 - lock_time  : date de validité de la signature (à voir avec sequence)