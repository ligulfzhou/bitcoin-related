use bitcoin::Network;
use btc::key_pair::AccountGenerator;
use electrum_client::{Client, ElectrumApi};

fn main() -> anyhow::Result<()> {
    dotenv::dotenv().unwrap();

    let mnemonic = std::env::var("MNEMONIC")?;
    let network = Network::from_core_arg(&std::env::var("NETWORK")?)?;

    let index = 0u32;

    let ag = AccountGenerator::new(&mnemonic, network)?;
    let account = ag.get_account_from_index(index)?;

    let script_pubkey = account.script_pubkey();
    let client = Client::new("tcp://127.0.0.1:50001")?;

    let utxos = client.script_list_unspent(script_pubkey.as_script())?;
    for utxo in utxos.iter() {
        println!("utxo: {:?}", utxo);
    }

    Ok(())
}
