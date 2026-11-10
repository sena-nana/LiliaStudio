use std::{fs, path::PathBuf, sync::Mutex};

use rusqlite::Connection;

use crate::db::{connection::open_database, migrations::run_migrations};

pub struct AppState {
    database: Mutex<Connection>,
}

impl AppState {
    pub fn initialize(app_data_dir: PathBuf) -> anyhow::Result<Self> {
        fs::create_dir_all(&app_data_dir)?;
        let db_path = app_data_dir.join("ameya.db");
        let mut connection = open_database(db_path)?;
        run_migrations(&mut connection)?;
        Ok(Self {
            database: Mutex::new(connection),
        })
    }

    pub fn with_database<T>(
        &self,
        action: impl FnOnce(&Connection) -> rusqlite::Result<T>,
    ) -> Result<T, String> {
        let connection = self
            .database
            .lock()
            .map_err(|_| "database lock is poisoned".to_string())?;
        action(&connection).map_err(|error| error.to_string())
    }
}
