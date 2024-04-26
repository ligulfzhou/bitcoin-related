use crate::key_pair::Account;
use bitcoin::{
    bip32::DerivationPath,
    key::KeyPair,
    secp256k1::{rand::thread_rng, Secp256k1, SecretKey},
    Network,
};

pub fn generate_keypair_randomly(network: Network) -> Account {
    let derivation_path = DerivationPath::default();

    let mut rng = thread_rng();
    let secp = Secp256k1::new();
    let secret_key = SecretKey::new(&mut rng);
    let keypair = KeyPair::from_secret_key(&secp, &secret_key);
    Account::new(keypair, network, derivation_path)
}
