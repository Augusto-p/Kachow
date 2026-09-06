pub mod contacts;
pub mod identity;
use rusqlite::{Connection};
use std::path::Path;
use std::sync::{Arc, Mutex};

pub struct Database{
    conn: Arc<Mutex<Connection>>,
}

impl Database{
     pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(path)?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS local_identity (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                secret_key BLOB NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                device_id TEXT NOT NULL,
                secret_service_name TEXT NOT NULL,
                display_name TEXT NOT NULL,
                tcp_port INTEGER NOT NULL,
                download_dir TEXT NOT NULL,
                device_image BLOB NOT NULL,
                mode BOOLEAN NOT NULL DEFAULT 0,
                public_from DATETIME,
                db_path TEXT NOT NULL,
                ipc_socket_path TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS contacts (
                device_id TEXT PRIMARY KEY,
                secret_service_name TEXT NOT NULL,
                display_name TEXT NOT NULL,
                public_key BLOB NOT NULL,
                device_image BLOB NOT NULL
            );
            "
        )?;

        Ok(())
    }
    pub fn connection(&self) -> Arc<Mutex<Connection>> {
        Arc::clone(&self.conn)
    }
}