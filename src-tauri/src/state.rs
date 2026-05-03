use rusqlite::Connection;
use std::collections::HashMap;
use std::sync::Mutex;

pub struct DbState(pub Mutex<Option<Connection>>);

pub struct CryptoState {
    pub key: Option<[u8; 32]>,
    pub salt: Option<Vec<u8>>,
}

pub type CryptoStateHandle = Mutex<CryptoState>;

pub struct ReplaceCache {
    pub ruleset_version: u64,
    pub entries: HashMap<u64, (String, u64)>,
}

pub type ReplaceCacheState = Mutex<ReplaceCache>;

pub fn unique_err(e: &rusqlite::Error, conflict_msg: &str) -> String {
    if e.to_string().contains("UNIQUE constraint failed") {
        conflict_msg.to_string()
    } else {
        e.to_string()
    }
}
