// use bdk::{
//     bitcoin::Network,
//     keys::{DerivableKey, ExtendedKey},
//     template::Bip86,
//     wallet::AddressIndex,
//     wallet::AddressInfo,
//     KeychainKind, Wallet,
// };
// use bip39::{Language, Mnemonic};
// use ordinals::Runestone;
// use secp256k1::Keypair;
//
// pub struct MyWallet<'a> {
//     pub wallet: Wallet,
//     mnemonic_code: &'a str,
// }
//
// impl<'a> MyWallet<'a> {
//     pub fn new(mnemonic_code: &'a str, network: Network) -> anyhow::Result<Self> {
//         let mnemonic = Mnemonic::parse_in_normalized(Language::English, mnemonic_code)?;
//         let xkey: ExtendedKey = mnemonic.into_extended_key().unwrap();
//         let xprv = xkey.into_xprv(network).unwrap();
//
//         let wallet = Wallet::new_no_persist(
//             Bip86(xprv, KeychainKind::External),
//             Some(Bip86(xprv, KeychainKind::Internal)),
//             network,
//         )?;
//
//         Ok(MyWallet {
//             wallet,
//             mnemonic_code,
//         })
//     }
//
//     pub fn get_keypair(&self, index: u32) -> Keypair {
//         // let signers = self.wallet.get_signers(KeychainKind::External);
//         // signers.signers().get(index);
//         todo!()
//     }
//
//     pub fn get_address_from_index(&mut self, index: u32) -> AddressInfo {
//         self.wallet.get_address(AddressIndex::Peek(index))
//     }
//
//     pub fn get_internal_address_from_index(&mut self, index: u32) -> AddressInfo {
//         self.wallet.get_internal_address(AddressIndex::Peek(index))
//     }
//
//     pub fn get_descriptor(&self, kind: KeychainKind) -> String {
//         self.wallet.get_descriptor_for_keychain(kind).to_string()
//     }
//
//     pub fn mnemonic_code(&self) -> &str {
//         self.mnemonic_code
//     }
// }
//
// // sign runestone transaction
// impl<'a> MyWallet<'a> {
//     pub fn sign_runestone_transaction(&self, runestone: &Runestone) {
//         todo!()
//     }
// }
//
// #[cfg(test)]
// mod tests {
//     use crate::key_pair::AccountGenerator;
//     use crate::wallet::MyWallet;
//     use bdk::bitcoin::Network;
//     use bdk::KeychainKind;
//
//     #[test]
//     fn test_gen_key_pair() {
//         dotenv::dotenv().unwrap();
//         let mnemonic_code =
//             "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
//         let network = Network::Bitcoin;
//         let mut my_wallet = MyWallet::new(mnemonic_code, network).unwrap();
//
//         let index_address = [
//             (
//                 0,
//                 "bc1p5cyxnuxmeuwuvkwfem96lqzszd02n6xdcjrs20cac6yqjjwudpxqkedrcr",
//             ),
//             (
//                 1,
//                 "bc1p4qhjn9zdvkux4e44uhx8tc55attvtyu358kutcqkudyccelu0was9fqzwh",
//             ),
//         ];
//
//         for (index, address) in index_address {
//             let addr = my_wallet.get_address_from_index(index);
//             assert_eq!(address, &addr.address.to_string());
//             println!("{}", addr.address.to_string());
//         }
//
//         println!(
//             "descriptor: {:?}",
//             my_wallet.get_descriptor(KeychainKind::External)
//         );
//
//         let ag = AccountGenerator::new(mnemonic_code, network).unwrap();
//         println!("{:?}", ag.fingerprint());
//         // assert_eq!(ag.fingerprint())
//     }
// }
