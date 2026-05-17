use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use parking_lot::Mutex;
use rusqlite::Connection;
use tauri::AppHandle;

use crate::db;
use crate::error::AppResult;

pub struct ImportControl {
    pub cancel: Arc<AtomicBool>,
}

pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
    pub data_dir: PathBuf,
    pub known_cards: Mutex<HashMap<String, super::device::Card>>,
    pub imports: Mutex<HashMap<String, ImportControl>>,
}

impl AppState {
    pub fn initialize(_handle: &AppHandle) -> AppResult<Self> {
        let data_dir = dirs::data_dir()
            .map(|d| d.join("cardgrab"))
            .unwrap_or_else(|| PathBuf::from("./cardgrab-data"));
        std::fs::create_dir_all(&data_dir)?;

        let db_path = data_dir.join("state.db");
        let conn = db::open(&db_path)?;

        Ok(Self {
            db: Arc::new(Mutex::new(conn)),
            data_dir,
            known_cards: Mutex::new(HashMap::new()),
            imports: Mutex::new(HashMap::new()),
        })
    }
}
