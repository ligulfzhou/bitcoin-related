use bitcoin::absolute::LockTime;
use bitcoin::{Network, OutPoint, Sequence, Transaction, TxIn, TxOut, Txid};
use bitcoin_private::hex::display::DisplayHex;
use btc::key_pair::AccountGenerator;
use electrum_client::{Client, ElectrumApi, ListUnspentRes};
use ordinals::{RuneId, Runestone};
use std::str::FromStr;

fn main() -> anyhow::Result<()> {
    dotenv::dotenv().unwrap();

    let mnemonic = std::env::var("MNEMONIC")?;
    let network = Network::from_core_arg(&std::env::var("NETWORK")?)?;

    let index = 0u32;

    let ag = AccountGenerator::new(&mnemonic, network)?;
    let account = ag.get_account_from_index(index)?;

    let runestone = Runestone {
        edicts: vec![],
        etching: None,
        mint: Some(RuneId::from_str("840000:52").unwrap()),
        pointer: Some(1),
    };

    let enc = runestone.encipher();
    println!("enc..{:?}", enc);

    // let script_pubkey = account.script_pubkey()?;
    let client = Client::new("tcp://127.0.0.1:50001")?;
    // let utxos = client.script_list_unspent(script_pubkey.as_script())?;

    // let utxos = vec![ListUnspentRes {
    //     height: 840229,
    //     tx_hash: Txid::from_str("021be35e2da16bf36a4476a8206c09823499ac16848c722a4c0f2fcc4316e382")
    //         .unwrap(),
    //     tx_pos: 1,
    //     value: 2036287,
    // }];

    let utxos = (0..5)
        .map(|i| ListUnspentRes {
            height: 840235,
            tx_hash: Txid::from_str(
                "2539044e1a75924e4724fa9e3053490aebfac860b37ada529099a0bc25a79d07",
            )
            .unwrap(),
            tx_pos: i,
            value: 394477,
        })
        .collect::<Vec<_>>();

    for utxo in utxos.iter() {
        println!("utxo: {:?}", utxo);
    }

    for utxo in utxos {
        let target_utxos = vec![utxo];
        println!("picked target_utxos: {:?}", &target_utxos);

        let input_value: u64 = target_utxos
            .iter()
            .map(|utxo| utxo.value)
            .collect::<Vec<_>>()
            .iter()
            .sum();

        let inputs = target_utxos
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

        let mut tx = Transaction {
            version: 2,
            lock_time: LockTime::ZERO,
            input: inputs, // pickup the utxos from step2.1, and convert them to txins
            output: vec![
                TxOut {
                    value: 0,
                    // runestone
                    script_pubkey: runestone.encipher(),
                },
                TxOut {
                    // change the value afterwards
                    value: 0,
                    // make left balance to itself
                    script_pubkey: account.script_pubkey(),
                },
            ],
        };
        let gas = tx.vsize() * 200;

        if input_value == 0 {
            println!("tx: {:?}", tx);
            return Ok(());
        }
        tx.output[1].value = input_value.checked_sub(gas as u64).unwrap();

        let signed_tx = ag.sign_tx(&tx, index, input_value)?;
        println!(
            "gas: {gas}, output_value: {}, signed_tx: {:?}",
            tx.output[1].value, signed_tx
        );

        println!(
            "hex: {:}",
            bitcoin::consensus::serialize(&signed_tx).to_lower_hex_string()
        );

        println!("---------------------------");

        let signed_hex = bitcoin::consensus::serialize(&signed_tx)
            .iter()
            .map(|u| format!("{:02X}", u))
            .collect::<Vec<_>>()
            .join("");
        println!("hex: {:}", signed_hex);
    }

    // let txid = client.transaction_broadcast(&signed_tx)?;
    // println!("runes txid: {:?}", txid);

    Ok(())
}
