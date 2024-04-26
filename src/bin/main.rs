use bitcoin::absolute::LockTime;
// use bitcoin::secp256k1::rand::Rng;
use bitcoin::{Network, OutPoint, Sequence, Transaction, TxIn, TxOut};
use btc::fee::get_recommended_fee;
use btc::key_pair::AccountGenerator;
use electrum_client::{Client, ElectrumApi};
use ordinals::{RuneId, Runestone};
// use secp256k1::rand::thread_rng;
use std::str::FromStr;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().unwrap();

    let mnemonic = std::env::var("MNEMONIC")?;
    let network = Network::from_core_arg(&std::env::var("NETWORK")?)?;

    let index = 0u32;

    let ag = AccountGenerator::new(&mnemonic, network)?;
    let account = ag.get_account_from_index(index)?;

    let script_pubkey = account.script_pubkey();
    let client = Client::new("tcp://127.0.0.1:50001")?;

    loop {
        let utxos = client.script_list_unspent(script_pubkey.as_script())?;
        let rf = get_recommended_fee().await?;
        let gas = {
            let cur = (rf.fastest_fee as f32 * 1.15) as u32;
            if cur > 115 {
                cur
            } else {
                115
            }
        };

        for utxo in utxos.iter() {
            if utxo.height == 0 {
                tokio::time::sleep(Duration::new(1, 0)).await;
                continue;
            }

            let rune_id = {
                // let n = thread_rng().gen_range(0u32..=1u32);
                // if n == 0 {
                //     "840202:2950"
                // } else {
                //     "840024:1404"
                // }

                "840024:1404"
            };
            let runestone = Runestone {
                edicts: vec![],
                etching: None,
                mint: Some(RuneId::from_str(rune_id).unwrap()),
                pointer: Some(1),
            };

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
            let gas = tx.vsize() * gas as usize;

            if input_value == 0 {
                println!("tx: {:?}", tx);
                return Ok(());
            }
            tx.output[1].value = input_value.checked_sub(gas as u64).unwrap(); // todo

            let signed_tx = ag.sign_tx(&tx, index, input_value)?;
            println!(
                "gas: {gas}, output_value: {}, signed_tx: {:?}",
                tx.output[1].value, signed_tx
            );

            tokio::time::sleep(Duration::new(5, 0)).await;
            let txid = client.transaction_broadcast(&signed_tx)?;
            println!("runes txid: {:?}", txid);
            println!("utxo: {:?}", utxo);
        }

        tokio::time::sleep(Duration::new(60, 0)).await;
    }
}
