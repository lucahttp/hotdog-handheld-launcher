//! Database module - SQLite storage for games and state

mod schema;
mod game_repository;
mod app_repository;

pub use schema::init_schema;
pub use game_repository::{GameRepository, Game};
pub use app_repository::{AppRepository, AppState};

use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use anyhow::Result;

/// Database manager
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    /// Open or create the database
    pub fn open() -> Result<Self> {
        let db_path = Self::db_path()?;
        
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        log::info!("Opening database at: {:?}", db_path);
        
        let conn = Connection::open(&db_path)?;
        conn.execute("PRAGMA foreign_keys = ON", [])?;
        
        let db = Self { conn: Arc::new(Mutex::new(conn)) };
        init_schema(&db.conn.lock().unwrap())?;
        
        Ok(db)
    }
    
    /// Get the database path
    fn db_path() -> Result<PathBuf> {
        let data_dir = dirs::data_local_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find local data directory"))?;
        
        Ok(data_dir.join("handheld-launcher").join("launcher.db"))
    }
    
    /// Get a game repository
    pub fn games(&self) -> GameRepository {
        GameRepository::new(Arc::clone(&self.conn))
    }
    
    /// Get an app repository
    pub fn app(&self) -> AppRepository {
        AppRepository::new(Arc::clone(&self.conn))
    }
}