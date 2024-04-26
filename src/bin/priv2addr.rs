use bitcoin::{PrivateKey, ScriptBuf};
use btc::key_pair::Account;
use std::str::FromStr;

fn main() -> anyhow::Result<()> {
    let script = ScriptBuf::from_hex("00142707085c54bb1bc236db9a7b5df11f892961237d")?;
    let privkey = PrivateKey::from_str("L4s7CSabyQ3ugknshjayE2qbkVNPQvrHmWYdq752ym4NnDeGVDVq")?;

    let account = Account::from(privkey);

    dbg!(script.to_string());

    dbg!(script.is_p2pkh());
    dbg!(account.p2pkh_address());
    dbg!(account.p2pkh_address_uncompressed());
    dbg!(script.is_p2sh());
    // dbg!(account.p2sh)
    // dbg!(script.is_p2pk());
    dbg!(script.is_v0_p2wpkh());
    dbg!(account.p2wpkh_address());
    dbg!(script.is_v0_p2wsh());
    dbg!(script.is_v1_p2tr());
    dbg!(account.p2tr_address());

    if script.is_p2pkh() {
        if account
            .p2pkh_address()?
            .script_pubkey()
            .as_script()
            .to_hex_string()
            .eq(&script.to_hex_string())
        {
            println!("address: {:?}", account.p2pkh_address()?.to_string());
        } else {
            println!(
                "address: {:?}",
                account.p2pkh_address_uncompressed()?.to_string()
            );
        }
    } else if script.is_v0_p2wpkh() {
        dbg!(".....");
        if account
            .p2wpkh_address()?
            .script_pubkey()
            .as_script()
            .to_hex_string()
            .eq(&script.to_hex_string())
        {
            println!(
                "address: {:?}",
                account.p2pkh_address_uncompressed()?.to_string()
            );
        }
    } else if script.is_v1_p2tr() {
        if account
            .p2tr_address()
            .script_pubkey()
            .as_script()
            .to_hex_string()
            .eq(&script.to_hex_string())
        {
            println!("address: {:?}", account.p2tr_address().to_string());
        }
    }

    Ok(())
}
