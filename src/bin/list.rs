use bitcoin::Network;
use btc::key_pair::AccountGenerator;
use dotenv::dotenv;

#[tokio::main]
async fn main() {
    dotenv().unwrap();
    let mnemonic_code = std::env::var("MNEMONIC").unwrap();
    let network = Network::from_core_arg(&std::env::var("NETWORK").unwrap()).unwrap();

    let ag = AccountGenerator::new(&mnemonic_code, network).unwrap();
    for idx in 0..=100u32 {
        let account = ag.get_account_from_index(idx).unwrap();
        println!(
            "{idx} {}, script: {:?}, hex: {:?}",
            account,
            account.script_pubkey().as_script().to_string(),
            account.script_pubkey().as_script().to_hex_string()
        );
    }
}
