use crate::key_pair::Account;
use electrum_client::{Client, ElectrumApi};

pub struct Electrs<'a> {
    url: &'a str,
    client: Client,
}

impl<'a> Electrs<'a> {
    pub fn new(url: &'a str) -> anyhow::Result<Self> {
        let client = Client::new("tcp://electrum.blockstream.info:50001")?;
        Ok(Self { url, client })
    }

    pub fn server_features(&self) -> anyhow::Result<()> {
        let response = self.client.server_features()?;

        println!("server_features: {:?}", response);

        Ok(())
    }

    pub fn get_list_utxo(&self, account: &Account) {
        // self.client.script_get_balance(account.tr_address())
        let script = account.p2tr_address().script_pubkey().as_script();
        // ScriptHash;
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::electrs::Electrs;

    #[test]
    fn test() {
        let electrs = Electrs::new("tcp://127.0.0.1:50001").unwrap();
        electrs.server_features().unwrap();
        // electrs.
    }
}
