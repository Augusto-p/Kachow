use std::{path::{Path, PathBuf}, sync::Arc};
use tokio::task;
use std::sync::{Mutex};
use rusqlite::Connection;


pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(path)?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        Ok(db)
    }

    pub async fn query_identity_single_field<T>(&self, query: &'static str) -> Option<T>
    where
        T: rusqlite::types::FromSql + Send + 'static,
    {
        let conn = Arc::clone(&self.conn);

        let result = task::spawn_blocking(move || {
            let conn = conn.lock().ok()?;
            let mut stmt = conn.prepare(query).ok()?;
            stmt.query_row([], |row| row.get(0)).ok()
        })
        .await;

        // Convierte JoinError en None y aplanar el Option interno
        result.ok().flatten()
    }
    pub async fn get_identity_ipc_socket_path(&self) -> Option<String> {
        self.query_identity_single_field("SELECT ipc_socket_path FROM local_identity WHERE id = 1")
            .await
    }
}
