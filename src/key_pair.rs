use anyhow::Error;
use bip39::{Language, Mnemonic};
use bitcoin::{
    bip32::{DerivationPath, ExtendedPrivKey, Fingerprint},
    hashes::{sha256, Hash},
    key::{KeyPair, TapTweak, XOnlyPublicKey},
    opcodes::{all, All},
    psbt::{Input, Psbt},
    script::Builder as SBuilder,
    secp256k1::{Secp256k1, SecretKey},
    sighash::{self, SighashCache, TapSighashType},
    taproot, Address, AddressType, Network, PrivateKey, PublicKey, ScriptBuf, Transaction, TxOut,
    Witness,
};
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Clone, Debug)]
pub struct Account {
    network: Network,
    keypair: KeyPair,
    derivation_path: DerivationPath,
}

impl Account {
    pub fn new(keypair: KeyPair, network: Network, derivation_path: DerivationPath) -> Self {
        Self {
            keypair,
            network,
            derivation_path,
        }
    }

    pub fn secret_key(&self) -> SecretKey {
        self.keypair.secret_key()
    }

    pub fn private_key(&self) -> anyhow::Result<PrivateKey> {
        let secret_key = self.keypair.secret_key();
        Ok(PrivateKey::new(secret_key, self.network))
    }

    pub fn private_key_uncompressed(&self) -> anyhow::Result<PrivateKey> {
        let secret_key = self.keypair.secret_key();
        Ok(PrivateKey::new_uncompressed(secret_key, self.network))
    }

    pub fn public_key(&self) -> anyhow::Result<PublicKey> {
        let secp = Secp256k1::new();
        let private_key = self.private_key()?;
        Ok(PublicKey::from_private_key(&secp, &private_key))
    }

    pub fn public_key_uncompressed(&self) -> anyhow::Result<PublicKey> {
        let secp = Secp256k1::new();
        let private_key = self.private_key_uncompressed()?;
        Ok(PublicKey::from_private_key(&secp, &private_key))
    }

    pub fn x_only_public_key(&self) -> XOnlyPublicKey {
        XOnlyPublicKey::from_keypair(&self.keypair).0
    }

    pub fn script_pubkey(&self) -> ScriptBuf {
        self.p2tr_address().script_pubkey()
    }

    pub fn script_hash(&self) -> String {
        let script_pubkey = self.script_pubkey();

        sha256::Hash::hash(script_pubkey.as_ref())[..]
            .iter()
            .rev()
            .map(|u| format!("{:x}", u))
            .collect::<Vec<String>>()
            .join("")
    }

    pub fn keypair(&self) -> KeyPair {
        self.keypair
    }

    pub fn derivation_path(&self) -> &DerivationPath {
        &self.derivation_path
    }

    pub fn network(&self) -> &Network {
        &self.network
    }
}

// address
impl Account {
    // pay to public key
    pub fn p2pkh_address(&self) -> anyhow::Result<Address> {
        Ok(Address::p2pkh(&self.public_key()?, self.network))
    }

    pub fn p2pkh_address_uncompressed(&self) -> anyhow::Result<Address> {
        Ok(Address::p2pkh(
            &self.public_key_uncompressed()?,
            self.network,
        ))
    }

    // pay-to-witness-pubkey-hash
    pub fn p2wpkh_address(&self) -> anyhow::Result<Address> {
        Ok(Address::p2wpkh(&self.public_key()?, self.network)?)
    }

    // pay to script address that embeds a witness pay to public key
    pub fn p2shwpkh_address(&self) -> anyhow::Result<Address> {
        Ok(Address::p2shwpkh(&self.public_key()?, self.network)?)
    }

    // pay-to-taproot
    pub fn p2tr_address(&self) -> Address {
        let secp = Secp256k1::new();
        let untweak_public_key = XOnlyPublicKey::from_keypair(&self.keypair);
        Address::p2tr(&secp, untweak_public_key.0, None, self.network)
    }

    // p2wsh
    // pub fn p2wsh_address(&self) -> anyhow::Result<Address> {
    //     let private_key = self.private_key()?;
    //
    //     todo!()
    // }
    // p2sh
    // p2shwsh
}

impl Account {
    pub fn transfer_btc_to_self(address_type: AddressType) {
        match address_type {
            AddressType::P2pkh => {}
            AddressType::P2tr => {}
            AddressType::P2wpkh => {}
            AddressType::P2wsh => {}
            _ => {}
        }
    }
}

impl From<PrivateKey> for Account {
    fn from(value: PrivateKey) -> Self {
        let secp = Secp256k1::new();

        let keypair = KeyPair::from_secret_key(&secp, &value.inner);
        Self {
            network: value.network,
            keypair,
            derivation_path: Default::default(),
        }
    }
}

impl From<SecretKey> for Account {
    fn from(value: SecretKey) -> Self {
        Account::from_secret_key(Network::Bitcoin, value)
    }
}

impl From<i128> for Account {
    fn from(value: i128) -> Self {
        Account::from_number(Network::Bitcoin, value)
    }
}

impl Account {
    pub fn from_number(network: Network, number: i128) -> Self {
        let secp = Secp256k1::new();

        let private_key_bytes = number.to_be_bytes();
        let mut padded_bytes = [0u8; 32];
        padded_bytes[private_key_bytes.len()..].copy_from_slice(&private_key_bytes);

        let secret_key =
            SecretKey::from_slice(&padded_bytes).expect("Failed to create SecretKey from bytes");
        let keypair = KeyPair::from_secret_key(&secp, &secret_key);

        Self {
            network,
            keypair,
            derivation_path: Default::default(),
        }
    }

    pub fn from_secret_key(network: Network, secret_key: SecretKey) -> Self {
        let secp = Secp256k1::new();
        let keypair = KeyPair::from_secret_key(&secp, &secret_key);

        Self {
            network,
            keypair,
            derivation_path: Default::default(),
        }
    }
}

impl From<u128> for Account {
    fn from(value: u128) -> Self {
        let secp = Secp256k1::new();

        let private_key_bytes = value.to_be_bytes();
        let mut padded_bytes = [0u8; 32];
        padded_bytes[private_key_bytes.len()..].copy_from_slice(&private_key_bytes);

        let secret_key =
            SecretKey::from_slice(&padded_bytes).expect("Failed to create SecretKey from bytes");
        let keypair = KeyPair::from_secret_key(&secp, &secret_key);
        Account::new(keypair, Network::Bitcoin, DerivationPath::default())
    }
}

impl Display for Account {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "privatekey: {:}, network:{:}, addresses: \n {:?}, \n {:?}, \n {:?} \n {:?} \n {:?}",
            self.private_key().unwrap().to_wif(),
            self.network.to_string(),
            self.p2pkh_address(),
            self.p2pkh_address_uncompressed(),
            self.p2wpkh_address(),
            self.p2shwpkh_address(),
            self.p2tr_address()
        )
    }
}

pub struct AccountGenerator<'a> {
    mnemonic_code: &'a str,
    seed: [u8; 64],
    network: Network,
    master_private_key: ExtendedPrivKey,
}

impl<'a> AccountGenerator<'a> {
    pub fn new(mnemonic_code: &'a str, network: Network) -> anyhow::Result<Self> {
        let mnemonic = Mnemonic::parse_in_normalized(Language::English, mnemonic_code)?;
        let seed = mnemonic.to_seed("");
        let master_private_key = ExtendedPrivKey::new_master(network, &seed)?;

        Ok(Self {
            mnemonic_code,
            seed,
            master_private_key,
            network,
        })
    }

    pub fn get_account_from_index(&self, index: u32) -> anyhow::Result<Account> {
        let secp = Secp256k1::new();
        let path = {
            let dp_prefix = match self.network {
                Network::Bitcoin => "m/86'/0'/0'/0",
                _ => "m/86'/1'/0'/0",
            };

            DerivationPath::from_str(&format!("{dp_prefix}/{index}")).unwrap()
        };

        let keypair = {
            let derived_key = self.master_private_key.derive_priv(&secp, &path).unwrap();
            let secret_key = SecretKey::from_slice(derived_key.private_key[..].as_ref())?;
            KeyPair::from_secret_key(&secp, &secret_key)
        };

        Ok(Account {
            network: self.network,
            keypair,
            derivation_path: path,
        })
    }

    pub fn sign_tx(
        &self,
        tx: &Transaction,
        idx: u32,
        input_value: u64,
    ) -> anyhow::Result<Transaction> {
        let secp = Secp256k1::new();

        let account = self.get_account_from_index(idx)?;
        let mut psbt = Psbt::from_unsigned_tx(tx.clone())?;
        let mut origins = BTreeMap::new();
        origins.insert(
            account.x_only_public_key(),
            (
                vec![],
                (self.fingerprint(), account.derivation_path().clone()),
            ),
        );

        let input = Input {
            witness_utxo: Some(TxOut {
                value: input_value,
                script_pubkey: account.script_pubkey(),
            }),
            sighash_type: Some(TapSighashType::All.into()),
            tap_internal_key: Some(account.x_only_public_key()),
            tap_key_origins: origins,
            ..Default::default()
        };

        psbt.inputs = vec![input];

        let unsigned_tx = psbt.unsigned_tx.clone();
        println!("unsigned_tx: {:?}", unsigned_tx);
        psbt.inputs
            .iter_mut()
            .enumerate()
            .try_for_each::<_, Result<(), Box<dyn std::error::Error>>>(|(vout, input)| {
                let hash_ty = input.sighash_type.unwrap().taproot_hash_ty()?;

                let hash = SighashCache::new(&unsigned_tx).taproot_key_spend_signature_hash(
                    vout,
                    &sighash::Prevouts::All(&[TxOut {
                        value: input_value,
                        script_pubkey: account.script_pubkey(),
                    }]),
                    hash_ty,
                )?;

                let keypair = KeyPair::from_seckey_slice(&secp, account.secret_key().as_ref())?;
                let keypair = keypair.tap_tweak(&secp, input.tap_merkle_root).to_inner();

                let sig = secp.sign_schnorr(&hash.into(), &keypair);
                let final_signature = taproot::Signature { sig, hash_ty };
                input.tap_key_sig = Some(final_signature);

                println!("tap_key_sig: {:?}", input.tap_key_sig);

                Ok(())
            })
            .unwrap();

        psbt.inputs.iter_mut().for_each(|input| {
            let mut script_witness: Witness = Witness::new();
            script_witness.push(input.tap_key_sig.unwrap().to_vec());
            input.final_script_witness = Some(script_witness);

            // Clear all the data fields as per the spec.
            input.partial_sigs = BTreeMap::new();
            input.sighash_type = None;
            input.redeem_script = None;
            input.witness_script = None;
            input.bip32_derivation = BTreeMap::new();
        });

        // EXTRACTOR
        let tx = psbt.extract_tx();
        println!("vsize: {:?}", tx.vsize());
        println!("tx: {:?}, ", tx);

        Ok(tx)
    }

    pub fn gen_n_of_n_multisig(&self, ids: &[u32], n: u32) -> anyhow::Result<ScriptBuf> {
        // m-of-n multisig, the n accounts are from ids
        if ids.len() as u32 != 3 {
            return Err(Error::msg("not valid"));
        }
        let pks = ids
            .iter()
            .map(|idx| self.get_account_from_index(*idx).unwrap())
            .map(|account| account.public_key().unwrap())
            .collect::<Vec<_>>();

        let op_push_num = All::from(all::OP_PUSHNUM_1.to_u8() + n as u8 - 1u8);
        let mut redeem_script = SBuilder::new().push_opcode(op_push_num);
        for pk in &pks {
            redeem_script = redeem_script.push_key(pk);
        }
        let redeemscript = redeem_script
            .push_opcode(op_push_num)
            .push_opcode(all::OP_CHECKMULTISIG)
            .into_script();

        Ok(redeemscript)
    }

    pub fn fingerprint(&self) -> Fingerprint {
        let secp = Secp256k1::new();
        self.master_private_key.fingerprint(&secp)
    }

    pub fn derivation_path(&self, index: u32) -> anyhow::Result<DerivationPath> {
        let dp_prefix = match self.network {
            Network::Bitcoin => "m/86'/0'/0'/0",
            _ => "m/86'/1'/0'/0",
        };

        Ok(DerivationPath::from_str(&format!("{dp_prefix}/{index}"))?)
    }

    pub fn mnemonic_code(&self) -> &str {
        self.mnemonic_code
    }

    pub fn seed(&self) -> [u8; 64] {
        self.seed
    }
}
