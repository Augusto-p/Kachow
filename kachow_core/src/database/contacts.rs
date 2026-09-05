use std::sync::Arc;

use base64::{engine::general_purpose, Engine as _};
use rusqlite::params;
use tokio::task;

use crate::database::Database;

#[derive(Debug, Clone)]
pub struct Contact {
    pub device_id: String,
    pub secret_service_name: String,
    pub display_name: String,
    pub public_key: Vec<u8>,
    pub device_image: Vec<u8>,
    pub last_seen_ip: Option<String>,
    pub last_seen_port: Option<u16>,
    pub last_seen_timestamp: Option<i64>,
    pub trust_level: String,
}

#[derive(Debug, Clone)]
pub struct ContactAddress {
    pub device_id: String,
    pub secret_service_name: String,
}

impl Database {
    /// Inserta un nuevo contacto o actualiza sus datos si ya existe (upsert)
    pub async fn set_contact(&self, contact: &Contact) -> bool {
        let conn = Arc::clone(&self.conn);
        let contact = contact.clone();

        let result = task::spawn_blocking(move || {
            let conn = conn.lock().ok()?;

            conn.execute(
                "INSERT INTO contacts (
                    device_id, secret_service_name, display_name, public_key, device_image,
                    last_seen_ip, last_seen_port, last_seen_timestamp, trust_level
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                ON CONFLICT(device_id) DO UPDATE SET
                    secret_service_name = excluded.secret_service_name,
                    display_name = excluded.display_name,
                    public_key = excluded.public_key,
                    device_image = excluded.device_image,
                    last_seen_ip = excluded.last_seen_ip,
                    last_seen_port = excluded.last_seen_port,
                    last_seen_timestamp = excluded.last_seen_timestamp,
                    trust_level = excluded.trust_level",
                params![
                    contact.device_id,
                    contact.secret_service_name,
                    contact.display_name,
                    contact.public_key,
                    contact.device_image,
                    contact.last_seen_ip,
                    contact.last_seen_port.map(|p| p as i32),
                    contact.last_seen_timestamp,
                    contact.trust_level,
                ],
            )
            .ok()
        })
        .await;

        result.ok().flatten().is_some()
    }

    /// Elimina un contacto por su device_id
    pub async fn delete_contact(&self, device_id: &str) -> bool {
        let conn = Arc::clone(&self.conn);
        let device_id = device_id.to_string();

        let result = task::spawn_blocking(move || {
            let conn = conn.lock().ok()?;
            let rows = conn
                .execute("DELETE FROM contacts WHERE device_id = ?1", params![device_id])
                .ok()?;
            Some(rows > 0)
        })
        .await;

        result.ok().flatten().unwrap_or(false)
    }

    /// Actualiza el estado de presencia (IP, Puerto, Timestamp)
pub async fn update_contact_presence(
        &self,
        device_id: &str,
        ip: Option<&str>,
        port: Option<u16>,
        timestamp: i64,
    ) -> bool {
        let conn = Arc::clone(&self.conn);
        let device_id = device_id.to_string();
        let ip = ip.map(|s| s.to_string());

        let result = task::spawn_blocking(move || {
            let conn = conn.lock().ok()?;
            let rows = conn
                .execute(
                    "UPDATE contacts 
                     SET last_seen_ip = ?1, last_seen_port = ?2, last_seen_timestamp = ?3 
                     WHERE device_id = ?4",
                    params![ip, port.map(|p| p as i32), timestamp, device_id],
                )
                .ok()?;
            Some(rows > 0)
        })
        .await;

        result.ok().flatten().unwrap_or(false)
    }

    /// Obtiene la lista completa de contactos
    pub async fn get_all_contacts(&self) -> Vec<Contact> {
        let conn = Arc::clone(&self.conn);

        let result = task::spawn_blocking(move || {
            let conn = conn.lock().ok()?;
            let mut stmt = conn
                .prepare(
                    "SELECT device_id, secret_service_name, display_name, public_key, 
                            device_image, last_seen_ip, last_seen_port, last_seen_timestamp, trust_level 
                     FROM contacts",
                )
                .ok()?;

            let rows = stmt
                .query_map([], |row| {
                    let port_i32: Option<i32> = row.get(6)?;
                    Ok(Contact {
                        device_id: row.get(0)?,
                        secret_service_name: row.get(1)?,
                        display_name: row.get(2)?,
                        public_key: row.get(3)?,
                        device_image: row.get(4)?,
                        last_seen_ip: row.get(5)?,
                        last_seen_port: port_i32.map(|p| p as u16),
                        last_seen_timestamp: row.get(7)?,
                        trust_level: row.get(8)?,
                    })
                })
                .ok()?;

            let mut contacts = Vec::new();
            for contact in rows.flatten() {
                contacts.push(contact);
            }
            Some(contacts)
        })
        .await;

        result.ok().flatten().unwrap_or_default()
    }

    /// Obtiene únicamente la lista de `device_id` junto a su `secret_service_name`
    pub async fn get_all_contact_addresses(&self) -> Vec<ContactAddress> {
        let conn = Arc::clone(&self.conn);

        let result = task::spawn_blocking(move || {
            let conn = conn.lock().ok()?;
            let mut stmt = conn
                .prepare("SELECT device_id, secret_service_name FROM contacts")
                .ok()?;

            let rows = stmt
                .query_map([], |row| {
                    Ok(ContactAddress {
                        device_id: row.get(0)?,
                        secret_service_name: row.get(1)?,
                    })
                })
                .ok()?;

            let mut addresses = Vec::new();
            for addr in rows.flatten() {
                addresses.push(addr);
            }
            Some(addresses)
        })
        .await;

        result.ok().flatten().unwrap_or_default()
    }

    /// Obtiene el display_name de un contacto por device_id
    pub async fn get_contact_display_name(&self, device_id: &str) -> Option<String> {
        self.query_contact_single_field(
            "SELECT display_name FROM contacts WHERE device_id = ?1",
            device_id,
        )
        .await
    }

    /// Obtiene la imagen de un contacto en formato Base64 por device_id
    pub async fn get_contact_device_image(&self, device_id: &str) -> Option<String> {
        if let Some(image_bytes) = self
            .query_contact_single_field::<Vec<u8>>(
                "SELECT device_image FROM contacts WHERE device_id = ?1",
                device_id,
            )
            .await
        {
            return Some(general_purpose::STANDARD.encode(&image_bytes));
        }
        None
    }

    /// Obtiene la clave pública de un contacto en bytes por device_id
    pub async fn get_contact_public_key(&self, device_id: &str) -> Option<Vec<u8>> {
        self.query_contact_single_field(
            "SELECT public_key FROM contacts WHERE device_id = ?1",
            device_id,
        )
        .await
    }

    /// Obtiene el secret_service_name de un contacto por device_id
    pub async fn get_contact_secret_service_name(&self, device_id: &str) -> Option<String> {
        self.query_contact_single_field(
            "SELECT secret_service_name FROM contacts WHERE device_id = ?1",
            device_id,
        )
        .await
    }

    //  Auxiliar interno para obtener un campo individual filtrando por device_id
    async fn query_contact_single_field<T>(&self, query: &'static str, device_id: &str) -> Option<T>
    where
        T: rusqlite::types::FromSql + Send + 'static,
    {
        let conn = Arc::clone(&self.conn);
        let device_id = device_id.to_string();

        let result = task::spawn_blocking(move || {
            let conn = conn.lock().ok()?;
            let mut stmt = conn.prepare(query).ok()?;
            stmt.query_row(params![device_id], |row| row.get(0)).ok()
        })
        .await;

        result.ok().flatten()
    }

    pub async fn has_contact(&self, device_id: &str) -> bool {
        let conn = Arc::clone(&self.conn);
        let device_id = device_id.to_string();

        let result = task::spawn_blocking(move || {
            let conn = conn.lock().ok()?;
            let mut stmt = conn
                .prepare("SELECT EXISTS(SELECT 1 FROM contacts WHERE device_id = ?1)")
                .ok()?;
            stmt.query_row(params![device_id], |row| row.get::<_, bool>(0))
                .ok()
        })
        .await;

        result.ok().flatten().unwrap_or(false)
    }
    /// Obtiene un contacto completo según su device_id
    pub async fn get_contact(&self, device_id: &str) -> Option<Contact> {
        let conn = Arc::clone(&self.conn);
        let device_id = device_id.to_string();

        let result = task::spawn_blocking(move || {
            let conn = conn.lock().ok()?;
            let mut stmt = conn
                .prepare(
                    "SELECT device_id, secret_service_name, display_name, public_key, 
                            device_image, last_seen_ip, last_seen_port, last_seen_timestamp, trust_level 
                     FROM contacts WHERE device_id = ?1",
                )
                .ok()?;

            stmt.query_row(params![device_id], |row| {
                let port_i32: Option<i32> = row.get(6)?;
                Ok(Contact {
                    device_id: row.get(0)?,
                    secret_service_name: row.get(1)?,
                    display_name: row.get(2)?,
                    public_key: row.get(3)?,
                    device_image: row.get(4)?,
                    last_seen_ip: row.get(5)?,
                    last_seen_port: port_i32.map(|p| p as u16),
                    last_seen_timestamp: row.get(7)?,
                    trust_level: row.get(8)?,
                })
            })
            .ok()
        })
        .await;

        result.ok().flatten()
    }
 }