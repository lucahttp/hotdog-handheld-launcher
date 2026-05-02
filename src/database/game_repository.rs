//! Game repository - CRUD operations for games

use anyhow::Result;
use rusqlite::{Connection, params};
use std::path::Path;

/// A game entry in the database
#[derive(Debug, Clone)]
pub struct Game {
    pub id: i64,
    pub title: String,
    pub path: String,
    pub icon_path: Option<String>,
    pub category: String,
    pub tile_size: String,  // "1x1", "2x1", "1x2"
    pub sort_order: i32,
}

impl Game {
    /// Create a new game
    pub fn new(title: &str, path: &str) -> Self {
        Self {
            id: 0,
            title: title.to_string(),
            path: path.to_string(),
            icon_path: None,
            category: "game".to_string(),
            tile_size: "1x1".to_string(),
            sort_order: 0,
        }
    }
    
    /// Check if the game executable exists
    pub fn exists(&self) -> bool {
        Path::new(&self.path).exists()
    }
}

/// Repository for game operations
pub struct GameRepository {
    conn: Connection,
}

impl GameRepository {
    /// Create a new repository
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }
    
    /// Insert a new game
    pub fn insert(&self, game: &Game) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO games (title, path, icon_path, category, tile_size, sort_order)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                game.title,
                game.path,
                game.icon_path,
                game.category,
                game.tile_size,
                game.sort_order,
            ],
        )?;
        
        Ok(self.conn.last_insert_rowid())
    }
    
    /// Update an existing game
    pub fn update(&self, game: &Game) -> Result<()> {
        self.conn.execute(
            "UPDATE games SET 
                title = ?1, path = ?2, icon_path = ?3, 
                category = ?4, tile_size = ?5, sort_order = ?6,
                updated_at = datetime('now')
             WHERE id = ?7",
            params![
                game.title,
                game.path,
                game.icon_path,
                game.category,
                game.tile_size,
                game.sort_order,
                game.id,
            ],
        )?;
        
        Ok(())
    }
    
    /// Delete a game
    pub fn delete(&self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM games WHERE id = ?1", params![id])?;
        Ok(())
    }
    
    /// Get a game by ID
    pub fn get_by_id(&self, id: i64) -> Result<Option<Game>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, path, icon_path, category, tile_size, sort_order
             FROM games WHERE id = ?1",
        )?;
        
        let mut rows = stmt.query(params![id])?;
        
        if let Some(row) = rows.next()? {
            Ok(Some(Game {
                id: row.get(0)?,
                title: row.get(1)?,
                path: row.get(2)?,
                icon_path: row.get(3)?,
                category: row.get(4)?,
                tile_size: row.get(5)?,
                sort_order: row.get(6)?,
            }))
        } else {
            Ok(None)
        }
    }
    
    /// Get all games
    pub fn get_all(&self) -> Result<Vec<Game>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, path, icon_path, category, tile_size, sort_order
             FROM games ORDER BY sort_order, title",
        )?;
        
        let mut rows = stmt.query([])?;
        let mut games = Vec::new();
        
        while let Some(row) = rows.next()? {
            games.push(Game {
                id: row.get(0)?,
                title: row.get(1)?,
                path: row.get(2)?,
                icon_path: row.get(3)?,
                category: row.get(4)?,
                tile_size: row.get(5)?,
                sort_order: row.get(6)?,
            });
        }
        
        Ok(games)
    }
    
    /// Get games by category
    pub fn get_by_category(&self, category: &str) -> Result<Vec<Game>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, path, icon_path, category, tile_size, sort_order
             FROM games WHERE category = ?1 ORDER BY sort_order, title",
        )?;
        
        let mut rows = stmt.query(params![category])?;
        let mut games = Vec::new();
        
        while let Some(row) = rows.next()? {
            games.push(Game {
                id: row.get(0)?,
                title: row.get(1)?,
                path: row.get(2)?,
                icon_path: row.get(3)?,
                category: row.get(4)?,
                tile_size: row.get(5)?,
                sort_order: row.get(6)?,
            });
        }
        
        Ok(games)
    }
    
    /// Search games by title
    pub fn search(&self, query: &str) -> Result<Vec<Game>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, path, icon_path, category, tile_size, sort_order
             FROM games WHERE title LIKE ?1 ORDER BY sort_order, title",
        )?;
        
        let search_pattern = format!("%{}%", query);
        let mut rows = stmt.query(params![search_pattern])?;
        let mut games = Vec::new();
        
        while let Some(row) = rows.next()? {
            games.push(Game {
                id: row.get(0)?,
                title: row.get(1)?,
                path: row.get(2)?,
                icon_path: row.get(3)?,
                category: row.get(4)?,
                tile_size: row.get(5)?,
                sort_order: row.get(6)?,
            });
        }
        
        Ok(games)
    }
    
    /// Add a sample game for testing
    pub fn add_sample_games(&self) -> Result<()> {
        let sample_games = vec![
            ("Halo Infinite", "C:\\Games\\halo_infinite.exe", "2x1"),
            ("Forza Horizon 5", "C:\\Games\\forza_horizon_5.exe", "1x1"),
            ("Sea of Thieves", "C:\\Games\\sea_of_thieves.exe", "1x1"),
            ("Emulator - RetroArch", "C:\\Emulators\\retroarch.exe", "1x1"),
            ("Settings", "C:\\Windows\\System32\\SettingsHost.exe", "1x1"),
        ];
        
        for (title, path, size) in sample_games {
            let game = Game {
                id: 0,
                title: title.to_string(),
                path: path.to_string(),
                icon_path: None,
                category: if title.contains("Emulator") { "emulator" } else { "game" }.to_string(),
                tile_size: size.to_string(),
                sort_order: 0,
            };
            self.insert(&game)?;
        }
        
        log::info!("Added {} sample games", sample_games.len());
        Ok(())
    }
}
