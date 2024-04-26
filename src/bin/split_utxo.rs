use bitcoin::absolute::LockTime;
use bitcoin::{Network, OutPoint, Sequence, Transaction, TxIn, TxOut, Txid};
use btc::key_pair::AccountGenerator;
use electrum_client::{Client, ElectrumApi, ListUnspentRes};
use std::str::FromStr;

fn main() -> anyhow::Result<()> {
    dotenv::dotenv().unwrap();

    let mnemonic = std::env::var("MNEMONIC")?;
    let network = Network::from_core_arg(&std::env::var("NETWORK")?)?;

    let index = 0u32;
    let splits = 5u64;

    let ag = AccountGenerator::new(&mnemonic, network)?;
    let account = ag.get_account_from_index(index)?;

    // let script_pubkey = account.script_pubkey();
    let client = Client::new("tcp://127.0.0.1:50001")?;

    // let utxos = client.script_list_unspent(script_pubkey.as_script())?;
    // for utxo in utxos.iter() {
    //     println!("utxo: {:?}", utxo);
    // }

    let utxos = vec![ListUnspentRes {
        height: 840234,
        tx_hash: Txid::from_str("021be35e2da16bf36a4476a8206c09823499ac16848c722a4c0f2fcc4316e382")
            .unwrap(),
        tx_pos: 1,
        value: 2036287,
    }];

    // let target_utxos = utxos
    //     .iter()
    //     .filter(|utxo| utxo.value > 10000000000000000)
    //     // .filter(|utxo| utxo.height > 0 && utxo.value > 6000 && utxo.value < 7000)
    //     .collect::<Vec<_>>();

    // println!("picked target_utxos: {:?}", &target_utxos);
    //
    // let input_values = target_utxos
    //     .iter()
    //     .map(|utxo| utxo.value)
    //     .collect::<Vec<_>>();

    // let input_value_sum = input_values.iter().sum();
    // let value = (input_value_sum - 160 * 50) / splits;
    // println!(
    //     "each got {:} sats, and gas is: {:}",
    //     value,
    //     input_value_sum - value * splits
    // );

    let utxo = utxos.first().unwrap();
    if utxo.value < 2036287 {
        return Ok(());
    }

    let value = (utxo.value - 225 * 284) / splits;

    println!(
        "each got {:} sats, and gas is: {:}",
        value,
        utxo.value - value * splits
    );

    let outputs = (0..splits)
        .map(|_| TxOut {
            value,
            script_pubkey: account.script_pubkey(),
        })
        .collect::<Vec<_>>();

    dbg!(outputs.len());

    let inputs = utxos
        .iter()
        .map(|utxo| TxIn {
            previous_output: OutPoint {
                txid: utxo.tx_hash,
                vout: utxo.tx_pos as u32,
            },
            script_sig: Default::default(),
            sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
            witness: Default::default(),
        })
        .collect::<Vec<_>>();

    let tx = Transaction {
        version: 1,
        lock_time: LockTime::ZERO,
        input: inputs, // pickup the utxos from step2.1, and convert them to txins
        output: outputs,
    };

    let signed_tx = ag.sign_tx(&tx, index, utxo.value)?;
    println!("signed tx: {:?}", signed_tx);

    let signed_hex = bitcoin::consensus::serialize(&signed_tx)
        .iter()
        .map(|u| format!("{:02X}", u))
        .collect::<Vec<_>>()
        .join("");
    println!("hex: {:}", signed_hex);

    // let txid = client.transaction_broadcast(&signed_tx)?;
    // println!("runes txid: {:?}", txid);

    Ok(())
}
