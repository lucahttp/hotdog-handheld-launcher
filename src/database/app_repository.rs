//! App state repository - Persists UI state

use anyhow::Result;
use rusqlite::{Connection, params};
use std::sync::{Arc, Mutex};

/// Application state
#[derive(Debug, Clone)]
pub struct AppState {
    pub last_focused_game: Option<i64>,
    pub last_tab: String,
}

/// Repository for app state operations
pub struct AppRepository {
    conn: Arc<Mutex<Connection>>,
}

impl AppRepository {
    /// Create a new repository
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }
    
    /// Get the current app state
    pub fn get_state(&self) -> Result<AppState> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT last_focused_game, last_tab FROM app_state WHERE id = 1",
        )?;
        
        let mut rows = stmt.query([])?;
        
        if let Some(row) = rows.next()? {
            Ok(AppState {
                last_focused_game: row.get(0)?,
                last_tab: row.get::<_, String>(1).unwrap_or_else(|_| "home".to_string()),
            })
        } else {
            // Return default state if not found
            Ok(AppState {
                last_focused_game: None,
                last_tab: "home".to_string(),
            })
        }
    }
    
    /// Save the app state
    pub fn save_state(&self, state: &AppState) -> Result<()> {
        self.conn.lock().unwrap().execute(
            "UPDATE app_state SET 
                last_focused_game = ?1, last_tab = ?2
             WHERE id = 1",
            params![state.last_focused_game, state.last_tab],
        )?;
        
        Ok(())
    }
    
    /// Update last focused game
    pub fn set_last_focused_game(&self, game_id: i64) -> Result<()> {
        self.conn.lock().unwrap().execute(
            "UPDATE app_state SET last_focused_game = ?1 WHERE id = 1",
            params![game_id],
        )?;
        Ok(())
    }
    
    /// Update last tab
    pub fn set_last_tab(&self, tab: &str) -> Result<()> {
        self.conn.lock().unwrap().execute(
            "UPDATE app_state SET last_tab = ?1 WHERE id = 1",
            params![tab],
        )?;
        Ok(())
    }
}
