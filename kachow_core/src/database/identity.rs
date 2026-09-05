use crate::identity::keys::IdentityKeyPair;
use crate::{database::Database, utils::Utils};
use base64::{engine::general_purpose, Engine as _};
use chrono::Local;
use rusqlite::params;
use std::{path::PathBuf, sync::Arc};
use tokio::task;

#[derive(Debug, Clone)]
pub struct LocalIdentity {
    pub id: i64,
    pub secret_key: Vec<u8>,
    pub secret_service_name: String,
    pub created_at: String,
    pub device_id: String,
    pub display_name: String,
    pub tcp_port: u16,
    pub download_dir: String,
    pub device_image: Vec<u8>,
    pub mode: bool,
    pub public_from: Option<String>,
    pub db_path: String,
    pub ipc_socket_path: String,
}

impl Default for LocalIdentity {
    fn default() -> Self {
        let data_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("kachow");

        let download_dir = dirs::download_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("kachowDownloads")
            .to_string_lossy()
            .into_owned(); // or .to_string()

        let runtime_dir = dirs::runtime_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        let db_path = data_dir.join("kachow.db").to_string_lossy().into_owned();
        let ipc_socket_path = runtime_dir
            .join("kachow.sock")
            .to_string_lossy()
            .into_owned();
        let device_image: Vec<u8> = Vec::new();

        let identity = IdentityKeyPair::generate();

        let identity_bytes = identity.to_bytes().to_vec();
        let device_id = identity.device_id();
        let secret_service_name = Utils::generate_random_string(20);

        Self {
            display_name: gethostname::gethostname().to_string_lossy().into_owned(),
            tcp_port: 9533,
            download_dir,
            db_path,
            ipc_socket_path,
            id: 1,
            secret_key: identity_bytes,
            secret_service_name: secret_service_name,
            created_at: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            device_id: device_id,
            device_image: device_image,
            mode: false,
            public_from: None,
        }
    }
}

impl Database {
    /// Obtiene toda la información de la identidad local (id = 1)
    pub async fn get_identity(&self) -> Option<LocalIdentity> {
        let conn = Arc::clone(&self.conn);

        let result = task::spawn_blocking(move || {
            let conn = conn.lock().ok()?;
            let mut stmt = conn
                .prepare(
                    "SELECT id, secret_key, datetime(created_at), device_id, display_name, 
                    tcp_port, download_dir, device_image, mode, datetime(public_from), 
                    secret_service_name, db_path, ipc_socket_path
             FROM local_identity WHERE id = 1",
                )
                .ok()?;

            stmt.query_row([], |row| {
                let port_i32: i32 = row.get(5)?;
                let mode_i32: i32 = row.get(8)?;

                Ok(LocalIdentity {
                    id: row.get(0)?,
                    secret_key: row.get(1)?,
                    created_at: row.get(2)?,
                    device_id: row.get(3)?,
                    display_name: row.get(4)?,
                    tcp_port: port_i32 as u16,
                    download_dir: row.get(6)?,
                    device_image: row.get(7)?,
                    mode: mode_i32 != 0,
                    public_from: row.get(9)?,
                    secret_service_name: row.get(10)?,
                    db_path: row.get(11)?,
                    ipc_socket_path: row.get(12)?,
                })
            })
            .ok()
        })
        .await;

        // Si spawn_blocking fue exitoso, desempaca el Option retornado por el hilo;
        // si falló el hilo (panic/cancellation), retorna None.
        result.ok().flatten()
    }

    /// Consultas de campos individuales
    pub async fn get_identity_display_name(&self) -> Option<String> {
        self.query_identity_single_field("SELECT display_name FROM local_identity WHERE id = 1")
            .await
    }

    // secret_service_name
    pub async fn get_identity_secret_service_name(&self) -> Option<String> {
        self.query_identity_single_field(
            "SELECT secret_service_name FROM local_identity WHERE id = 1",
        )
        .await
    }

    pub async fn get_identity_ipc_socket_path(&self) -> Option<String> {
        self.query_identity_single_field("SELECT ipc_socket_path FROM local_identity WHERE id = 1")
            .await
    }

    pub async fn get_identity_db_path(&self) -> Option<String> {
        self.query_identity_single_field("SELECT db_path FROM local_identity WHERE id = 1")
            .await
    }
    pub async fn get_identity_mode(&self) -> Option<bool> {
        let mode: Option<i32> = self
            .query_identity_single_field("SELECT mode FROM local_identity WHERE id = 1")
            .await?;
        let mode_unwrap = mode.unwrap_or(0);
        if mode_unwrap == 0 {
            return Some(false);
        } else {
            return Some(true);
        }
    }

    pub async fn get_identity_public_from(&self) -> Option<String> {
        self.query_identity_single_field(
            "SELECT datetime(public_from) FROM local_identity WHERE id = 1",
        )
        .await
    }

    pub async fn get_identity_tcp_port(&self) -> Option<u16> {
        let port: Option<i32> = self
            .query_identity_single_field("SELECT tcp_port FROM local_identity WHERE id = 1")
            .await?;
        let port_unwrap = port.unwrap_or(9533);
        return Some(port_unwrap as u16);
    }

    pub async fn get_identity_download_dir(&self) -> Option<String> {
        self.query_identity_single_field("SELECT download_dir FROM local_identity WHERE id = 1")
            .await
    }

    pub async fn get_identity_device_image(&self) -> Option<String> {
        if let Some(image) = self
            .query_identity_single_field::<Vec<u8>>(
                "SELECT device_image FROM local_identity WHERE id = 1",
            )
            .await
        {
            return Some(general_purpose::STANDARD.encode(&image));
        }

        None
    }

    pub async fn get_identity_device_id(&self) -> Option<String> {
        self.query_identity_single_field("SELECT device_id FROM local_identity WHERE id = 1")
            .await
    }

    pub async fn get_identity_secret_key(&self) -> Option<IdentityKeyPair> {
    let secret_bytes = self
        .query_identity_single_field::<Vec<u8>>(
            "SELECT secret_key FROM local_identity WHERE id = 1"
        )
        .await?;

    let secret_bytes: [u8; 32] = secret_bytes.try_into().ok()?;

    Some(IdentityKeyPair::from_bytes(&secret_bytes))
}

    pub async fn update_identity_config(
        &self,
        display_name: &str,
        tcp_port: u16,
        download_dir: &str,
        device_image: &[u8],
        mode: bool,
    ) -> bool {
        let conn = Arc::clone(&self.conn);
        let display_name = display_name.to_string();
        let download_dir = download_dir.to_string();
        let device_image = device_image.to_vec();

        let result = task::spawn_blocking(move || {
            let conn = conn.lock().ok()?;
            let count = conn
                .execute(
                    "UPDATE local_identity 
                 SET display_name = ?1, 
                     tcp_port = ?2, 
                     download_dir = ?3, 
                     device_image = ?4,
                     mode = ?5,
                     public_from = CASE WHEN ?5 = 1 THEN CURRENT_TIMESTAMP ELSE NULL END
                 WHERE id = 1",
                    params![
                        display_name,
                        tcp_port as i32,
                        download_dir,
                        device_image,
                        mode as i32
                    ],
                )
                .ok()?;

            Some(count > 0)
        })
        .await;

        // Retorna true solo si la tarea se ejecutó sin errores y count > 0.
        result.ok().flatten().unwrap_or(false)
    }
    /// Actualizaciones individuales
    pub async fn set_identity_display_name(&self, name: &str) -> bool {
        let name = name.to_string();
        self.execute_identity_update(
            "UPDATE local_identity SET display_name = ?1 WHERE id = 1",
            (name,),
        )
        .await
    }

    pub async fn set_identity_tcp_port(&self, port: u16) -> bool {
        self.execute_identity_update(
            "UPDATE local_identity SET tcp_port = ?1 WHERE id = 1",
            (port as i32,),
        )
        .await
    }

    pub async fn set_identity_download_dir(&self, dir: &str) -> bool {
        let dir = dir.to_string();
        self.execute_identity_update(
            "UPDATE local_identity SET download_dir = ?1 WHERE id = 1",
            (dir,),
        )
        .await
    }

    pub async fn set_identity_device_image(&self, image: &[u8]) -> bool {
        let image = image.to_vec();
        self.execute_identity_update(
            "UPDATE local_identity SET device_image = ?1 WHERE id = 1",
            (image,),
        )
        .await
    }

    pub async fn set_identity(&self, identity: &LocalIdentity) -> bool {
        let conn = Arc::clone(&self.conn);
        let identity = identity.clone();

        let result = task::spawn_blocking(move || {
            let conn = conn.lock().ok()?;

            conn.execute(
                "INSERT INTO local_identity (
            id, secret_key, device_id, display_name, tcp_port, download_dir, 
            device_image, mode, public_from, secret_service_name, db_path, ipc_socket_path
        ) VALUES (
            1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, 
            CASE WHEN ?7 = 1 THEN CURRENT_TIMESTAMP ELSE NULL END,
            ?8, ?9, ?10
        )
        ON CONFLICT(id) DO UPDATE SET
            secret_key = excluded.secret_key,
            device_id = excluded.device_id,
            display_name = excluded.display_name,
            tcp_port = excluded.tcp_port,
            download_dir = excluded.download_dir,
            device_image = excluded.device_image,
            mode = excluded.mode,
            public_from = CASE WHEN excluded.mode = 1 THEN CURRENT_TIMESTAMP ELSE NULL END,
            secret_service_name = excluded.secret_service_name,
            db_path = excluded.db_path,
            ipc_socket_path = excluded.ipc_socket_path",
                params![
                    identity.secret_key,          // ?1
                    identity.device_id,           // ?2
                    identity.display_name,        // ?3
                    identity.tcp_port as i32,     // ?4
                    identity.download_dir,        // ?5
                    identity.device_image,        // ?6
                    identity.mode as i32,         // ?7
                    identity.secret_service_name, // ?8
                    identity.db_path,             // ?9
                    identity.ipc_socket_path,     // ?10
                ],
            )
            .ok()
        })
        .await;

        // Devuelve true si la tarea completó correctamente y SQLite ejecutó la consulta
        result.ok().flatten().is_some()
    }

    pub async fn set_identity_mode(&self, mode: bool) -> bool {
        let conn = Arc::clone(&self.conn);

        let result = task::spawn_blocking(move || {
            let conn = conn.lock().ok()?;
            let count = conn
                .execute(
                    "UPDATE local_identity 
                 SET mode = ?1, 
                     public_from = CASE WHEN ?1 = 1 THEN CURRENT_TIMESTAMP ELSE NULL END 
                 WHERE id = 1",
                    params![mode as i32],
                )
                .ok()?;

            Some(count > 0)
        })
        .await;

        result.ok().flatten().unwrap_or(false)
    }

    /// Función auxiliar interna para extraer un único campo
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

    /// Función auxiliar para ejecutar sentencias UPDATE
    async fn execute_identity_update(
        &self,
        query: &'static str,
        params: impl rusqlite::Params + Send + 'static,
    ) -> bool {
        let conn = Arc::clone(&self.conn);

        let result = task::spawn_blocking(move || {
            let conn = conn.lock().ok()?;
            let rows_affected = conn.execute(query, params).ok()?;
            Some(rows_affected > 0)
        })
        .await;

        // Retorna true solo si la tarea se ejecutó correctamente y rows_affected > 0.
        // Cualquier fallo (mutex, SQL, join) resulta en false.
        result.ok().flatten().unwrap_or(false)
    }
}
