use std::{collections::HashMap, sync::{Arc, atomic::{AtomicBool, Ordering}}};

use tokio::sync::RwLock;

use crate::{database::Database, identity::pair_key::PairKey};

pub struct KachowState {
    pub storage: Arc<Database>,
    pub pair_key: Arc<PairKey>,
    pub mode: Arc<AtomicBool>, // Garantiza mutación concurrente segura
    pub discovered: RwLock<HashMap<String, String>>,
}

impl KachowState {
    pub fn new(storage: Arc<Database>) -> Self {
        Self {
            storage,
            pair_key: Arc::new(PairKey::new()),
            mode: Arc::new(AtomicBool::new(false)),
            discovered: RwLock::new(HashMap::new()), // Cambiado Vec::new() por HashMap::new()
        }
    }

    /// Cambia el modo y gestiona la clave y el temporizador en consecuencia
    pub async fn change_mode(&self, new_mode: bool) {
        self.mode.store(new_mode, Ordering::SeqCst);

        if new_mode {
            let mode_clone = Arc::clone(&self.mode);
            self.storage.set_identity_mode(true);

            // Genera la clave e inicia los 10 minutos
            let storage_clone = self.storage.clone();
            self.pair_key.generate_and_start_timer(move || {
                // Callback ejecutado tras 10 minutos
                mode_clone.store(false, Ordering::SeqCst);
                storage_clone.set_identity_mode(false);
                println!("Modo desactivado automáticamente tras 10 minutos.");
            });
        } else {
            // Si el modo se cambia a false manualmente, cancelamos todo
            self.pair_key.clear();
            self.storage.set_identity_mode(false);
        }
    }

    pub fn is_mode_active(&self) -> bool {
        self.mode.load(Ordering::SeqCst)
    }

    /// Inserta o actualiza un dispositivo (Key -> Value)
    pub async fn add_discovered_device(&self, key: String, value: String) {
        let mut lock = self.discovered.write().await;
        lock.insert(key, value);
    }

    /// Elimina un dispositivo por su clave
    pub async fn remove_discovered_device(&self, key: &str) {
        let mut lock = self.discovered.write().await;
        lock.remove(key);
    }

    /// Devuelve un Vec con todas las claves (Keys)
    pub async fn get_discovered_devices(&self) -> Vec<String> {
        let lock = self.discovered.read().await;
        lock.keys().cloned().collect()
    }

    /// Obtiene el valor asociado a una clave específica
    pub async fn get_device_value(&self, key: &str) -> Option<String> {
        let lock = self.discovered.read().await;
        lock.get(key).cloned()
    }
}