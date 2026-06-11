use std::path::PathBuf;
use log::{info, warn};
use rusqlite::{Connection, params};
use crate::commands::editing::ProjectInfo;

const DB_FILE_NAME: &str = ".aether/aether.db";

#[derive(Debug)]
pub struct ProjectRegistry {
    conn: Connection,
}

impl ProjectRegistry {
    pub fn new() -> Result<Self, String> {
        let db_path = Self::db_path();
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create database directory: {}", e))?;
        }
        let conn = Connection::open(&db_path)
            .map_err(|e| format!("Failed to open database: {}", e))?;
        let registry = Self { conn };
        registry.init_schema()?;
        info!("Project registry SQLite db opened at {:?}", db_path);
        Ok(registry)
    }

    fn db_path() -> PathBuf {
        home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(DB_FILE_NAME)
    }

    fn init_schema(&self) -> Result<(), String> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                created_at TEXT NOT NULL,
                modified_at TEXT NOT NULL,
                duration REAL NOT NULL DEFAULT 0,
                fps REAL NOT NULL DEFAULT 30,
                width INTEGER NOT NULL DEFAULT 1920,
                height INTEGER NOT NULL DEFAULT 1080,
                timeline_count INTEGER NOT NULL DEFAULT 1,
                media_count INTEGER NOT NULL DEFAULT 0,
                file_size INTEGER NOT NULL DEFAULT 0,
                file_path TEXT NOT NULL
            )",
            [],
        ).map_err(|e| format!("Failed to create projects table: {}", e))?;
        Ok(())
    }

    pub fn add(&self, project: &ProjectInfo) -> Result<(), String> {
        self.conn.execute(
            "INSERT OR REPLACE INTO projects
             (id, name, description, created_at, modified_at, duration, fps, width, height, timeline_count, media_count, file_size, file_path)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                project.id,
                project.name,
                project.description,
                project.created_at,
                project.modified_at,
                project.duration,
                project.fps,
                project.resolution.0 as i64,
                project.resolution.1 as i64,
                project.timeline_count as i64,
                project.media_count as i64,
                project.file_size as i64,
                project.file_path,
            ],
        ).map_err(|e| format!("Failed to insert project: {}", e))?;
        info!("Registered project {} in SQLite registry", project.id);
        Ok(())
    }

    pub fn remove(&self, project_id: &str) -> Result<bool, String> {
        let rows = self.conn.execute(
            "DELETE FROM projects WHERE id = ?1",
            [project_id],
        ).map_err(|e| format!("Failed to remove project: {}", e))?;
        Ok(rows > 0)
    }

    pub fn get(&self, project_id: &str) -> Result<Option<ProjectInfo>, String> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, description, created_at, modified_at, duration, fps, width, height, timeline_count, media_count, file_size, file_path
             FROM projects WHERE id = ?1"
        ).map_err(|e| format!("Failed to prepare query: {}", e))?;

        let mut rows = stmt.query([project_id])
            .map_err(|e| format!("Failed to query project: {}", e))?;

        if let Some(row) = rows.next()
            .map_err(|e| format!("Failed to read row: {}", e))? {
            Ok(Some(Self::row_to_project(row).map_err(|e| format!("Row parse error: {}", e))?))
        } else {
            Ok(None)
        }
    }

    pub fn get_all(&self) -> Result<Vec<ProjectInfo>, String> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, description, created_at, modified_at, duration, fps, width, height, timeline_count, media_count, file_size, file_path
             FROM projects ORDER BY modified_at DESC"
        ).map_err(|e| format!("Failed to prepare query: {}", e))?;

        let rows = stmt.query_map([], |row| Self::row_to_project(row))
            .map_err(|e| format!("Failed to query projects: {}", e))?;

        let mut projects = Vec::new();
        for row in rows {
            match row {
                Ok(p) => projects.push(p),
                Err(e) => warn!("Skipping invalid project row: {}", e),
            }
        }
        Ok(projects)
    }

    pub fn get_recent(&self, limit: usize) -> Result<Vec<ProjectInfo>, String> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, description, created_at, modified_at, duration, fps, width, height, timeline_count, media_count, file_size, file_path
             FROM projects ORDER BY modified_at DESC LIMIT ?1"
        ).map_err(|e| format!("Failed to prepare query: {}", e))?;

        let rows = stmt.query_map([limit as i64], |row| Self::row_to_project(row))
            .map_err(|e| format!("Failed to query recent projects: {}", e))?;

        let mut projects = Vec::new();
        for row in rows {
            match row {
                Ok(p) => projects.push(p),
                Err(e) => warn!("Skipping invalid project row: {}", e),
            }
        }
        Ok(projects)
    }

    fn row_to_project(row: &rusqlite::Row) -> Result<ProjectInfo, rusqlite::Error> {
        Ok(ProjectInfo {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            created_at: row.get(3)?,
            modified_at: row.get(4)?,
            duration: row.get(5)?,
            fps: row.get(6)?,
            resolution: (row.get::<_, i64>(7)? as u32, row.get::<_, i64>(8)? as u32),
            timeline_count: row.get::<_, i64>(9)? as usize,
            media_count: row.get::<_, i64>(10)? as usize,
            file_size: row.get::<_, i64>(11)? as u64,
            file_path: row.get(12)?,
        })
    }
}

impl Default for ProjectRegistry {
    fn default() -> Self {
        Self::new().expect("Failed to create default ProjectRegistry")
    }
}

fn home_dir() -> Option<PathBuf> {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .ok()
}
