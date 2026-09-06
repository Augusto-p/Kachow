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
}

#[derive(Debug, Clone)]
pub struct ContactAddress {
    pub device_id: String,
    pub secret_service_name: String,
}

impl Database {
    /// Inserta o actualiza un contacto en la base de datos (Upsert)
    pub async fn set_contact(&self, contact: &Contact) -> bool {
        let conn = Arc::clone(&self.conn);
        let contact = contact.clone();

        let result = task::spawn_blocking(move || {
            let conn = conn.lock().ok()?;

            conn.execute(
                "INSERT INTO contacts (
                    device_id, secret_service_name, display_name, public_key, device_image
                ) VALUES (?1, ?2, ?3, ?4, ?5)
                ON CONFLICT(device_id) DO UPDATE SET
                    secret_service_name = excluded.secret_service_name,
                    display_name = excluded.display_name,
                    public_key = excluded.public_key,
                    device_image = excluded.device_image",
                params![
                    contact.device_id,
                    contact.secret_service_name,
                    contact.display_name,
                    contact.public_key,
                    contact.device_image,
                ],
            )
            .ok()
        })
        .await;

        result.ok().flatten().is_some()
    }

    /// Obtiene un contacto completo según su device_id
    pub async fn get_contact(&self, device_id: &str) -> Option<Contact> {
        let conn = Arc::clone(&self.conn);
        let device_id = device_id.to_string();

        let result = task::spawn_blocking(move || {
            let conn = conn.lock().ok()?;
            let mut stmt = conn
                .prepare(
                    "SELECT device_id, secret_service_name, display_name, public_key, device_image 
                     FROM contacts WHERE device_id = ?1",
                )
                .ok()?;

            stmt.query_row(params![device_id], |row| {
                Ok(Contact {
                    device_id: row.get(0)?,
                    secret_service_name: row.get(1)?,
                    display_name: row.get(2)?,
                    public_key: row.get(3)?,
                    device_image: row.get(4)?,
                })
            })
            .ok()
        })
        .await;

        result.ok().flatten()
    }

    /// Comprueba si existe un contacto por su device_id
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

    /// Elimina un contacto por su device_id
    pub async fn delete_contact(&self, device_id: &str) -> bool {
        let conn = Arc::clone(&self.conn);
        let device_id = device_id.to_string();

        let result = task::spawn_blocking(move || {
            let conn = conn.lock().ok()?;
            let rows = conn
                .execute(
                    "DELETE FROM contacts WHERE device_id = ?1",
                    params![device_id],
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
                    "SELECT device_id, secret_service_name, display_name, public_key, device_image 
                     FROM contacts",
                )
                .ok()?;

            let rows = stmt
                .query_map([], |row| {
                    Ok(Contact {
                        device_id: row.get(0)?,
                        secret_service_name: row.get(1)?,
                        display_name: row.get(2)?,
                        public_key: row.get(3)?,
                        device_image: row.get(4)?,
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

    /// Obtiene todos los secret_service_name junto a su device_id
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

    /// Consultas de campos individuales
    pub async fn get_contact_display_name(&self, device_id: &str) -> Option<String> {
        self.query_contact_single_field(
            "SELECT display_name FROM contacts WHERE device_id = ?1",
            device_id,
        )
        .await
    }

    pub async fn get_contact_secret_service_name(&self, device_id: &str) -> Option<String> {
        self.query_contact_single_field(
            "SELECT secret_service_name FROM contacts WHERE device_id = ?1",
            device_id,
        )
        .await
    }

    pub async fn get_contact_public_key(&self, device_id: &str) -> Option<Vec<u8>> {
        self.query_contact_single_field(
            "SELECT public_key FROM contacts WHERE device_id = ?1",
            device_id,
        )
        .await
    }

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

    /// Auxiliar interno para obtener un campo individual por device_id
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

    /// Actualiza únicamente el nombre visible (display_name) de un contacto
    pub async fn update_contact_display_name(&self, device_id: &str, display_name: &str) -> bool {
        let display_name = display_name.to_string();
        self.execute_contact_update(
            "UPDATE contacts SET display_name = ?1 WHERE device_id = ?2",
            display_name,
            device_id,
        )
        .await
    }

    /// Actualiza únicamente la imagen (device_image) de un contacto
    pub async fn update_contact_device_image(&self, device_id: &str, image: &[u8]) -> bool {
        let image = image.to_vec();
        self.execute_contact_update(
            "UPDATE contacts SET device_image = ?1 WHERE device_id = ?2",
            image,
            device_id,
        )
        .await
    }

    /// Función auxiliar interna para ejecutar sentencias UPDATE sobre un contacto específico
    async fn execute_contact_update(
        &self,
        query: &'static str,
        value: impl rusqlite::ToSql + Send + 'static,
        device_id: &str,
    ) -> bool {
        let conn = Arc::clone(&self.conn);
        let device_id = device_id.to_string();

        let result = task::spawn_blocking(move || {
            let conn = conn.lock().ok()?;
            let rows_affected = conn.execute(query, params![value, device_id]).ok()?;
            Some(rows_affected > 0)
        })
        .await;

        result.ok().flatten().unwrap_or(false)
    }
}
