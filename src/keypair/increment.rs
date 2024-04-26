use crate::key_pair::Account;
use bitcoin::Network;

pub fn generate_keypair_from_number(network: Network, number: i128) -> Account {
    Account::from_number(network, number)
}
