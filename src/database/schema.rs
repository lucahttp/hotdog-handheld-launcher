//! Database schema - Table definitions and migrations

use anyhow::Result;
use rusqlite::{Connection, params};

/// Database schema version
pub const SCHEMA_VERSION: i32 = 1;

/// Initialize the database schema
pub fn init_schema(conn: &Connection) -> Result<()> {
    log::info!("Initializing database schema v{}", SCHEMA_VERSION);
    
    // Games table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS games (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            path TEXT NOT NULL,
            icon_path TEXT,
            category TEXT DEFAULT 'game',
            tile_size TEXT DEFAULT '1x1',
            sort_order INTEGER DEFAULT 0,
            created_at TEXT DEFAULT (datetime('now')),
            updated_at TEXT DEFAULT (datetime('now'))
        )",
        [],
    )?;
    
    // Settings table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    )?;
    
    // App state table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS app_state (
            id INTEGER PRIMARY KEY,
            last_focused_game INTEGER,
            last_tab TEXT DEFAULT 'home',
            FOREIGN KEY (last_focused_game) REFERENCES games(id)
        )",
        [],
    )?;
    
    // Insert default app state if not exists
    conn.execute(
        "INSERT OR IGNORE INTO app_state (id, last_tab) VALUES (1, 'home')",
        [],
    )?;
    
    log::info!("Database schema initialized successfully");
    Ok(())
}
