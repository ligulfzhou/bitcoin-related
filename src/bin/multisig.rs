use bitcoin::{Address, Network};
use btc::key_pair::AccountGenerator;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().unwrap();
    let mnemonic_code = std::env::var("MNEMONIC").unwrap();
    let network = Network::from_core_arg(&std::env::var("NETWORK").unwrap()).unwrap();

    let ag = AccountGenerator::new(&mnemonic_code, network).unwrap();

    let scriptbuf = ag.gen_n_of_n_multisig(&[100, 101, 102], 3)?;
    println!("script buf: {:?}", scriptbuf);

    let address = Address::p2wsh(&scriptbuf, network);
    println!("multisig address: {:?}", address);
    Ok(())
}
