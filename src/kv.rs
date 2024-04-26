// use rocksdb::DB;
// use std::sync::Arc;
//
// pub trait KVStore {
//     fn init(file_path: &str) -> Self;
//     fn save(&self, k: &str, v: &str) -> bool;
//     fn find(&self, k: &str) -> Option<String>;
//     fn delete(&self, k: &str) -> bool;
// }
//
// pub struct RocksDB {
//     db: Arc<DB>,
// }
//
// impl KVStore for RocksDB {
//     fn init(file_path: &str) -> Self {
//         Self {
//             db: Arc::new(DB::open_default(file_path).unwrap()),
//         }
//     }
//
//     fn save(&self, k: &str, v: &str) -> bool {
//         self.db.put(k.as_bytes(), v.as_bytes()).is_ok()
//     }
//
//     fn find(&self, k: &str) -> Option<String> {
//         match self.db.get(k.as_bytes()) {
//             Ok(Some(value)) => Some(String::from_utf8(value).unwrap()),
//             _ => None,
//         }
//     }
//
//     fn delete(&self, k: &str) -> bool {
//         self.db.delete(k.as_bytes()).is_ok()
//     }
// }
//
// // specific fns
// impl RocksDB {
//     fn save_mnemonic(&self, mnemonic: &str) {
//         self.save(mnemonic, "");
//     }
// }
