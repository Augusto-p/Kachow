use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::state::KachowState;

pub struct MdnsManager {
    daemon: ServiceDaemon,
    current_fullnames: Arc<Mutex<Vec<String>>>,
    service_type: String, // Ejemplo: "_kachow._tcp.local."
    port: u16,
}

impl MdnsManager {
    pub fn new(service_type: &str, port: u16) -> Result<Self, Box<dyn std::error::Error>> {
        let daemon = ServiceDaemon::new()?;
        let normalized_type = Self::normalize_service_type(service_type);

        Ok(Self {
            daemon,
            current_fullnames: Arc::new(Mutex::new(Vec::new())),
            service_type: normalized_type,
            port,
        })
    }

    pub fn normalize_service_type(service_type: &str) -> String {
        let trimmed = service_type.trim_matches('.');
        if trimmed.ends_with("_tcp.local") || trimmed.ends_with("_udp.local") {
            format!("{}.", trimmed)
        } else {
            format!("_{}._tcp.local.", trimmed)
        }
    }

    /// Anuncia una lista de nombres de instancia bajo el service_type principal
    pub fn announce_names(&self, instance_names: &[String]) -> Result<(), Box<dyn std::error::Error>> {
        let mut current = self.current_fullnames.lock().unwrap();

        // 1. Limpiar registros viejos
        for old_fullname in current.drain(..) {
            let _ = self.daemon.unregister(&old_fullname);
        }

        // 2. Registrar los nuevos nombres de instancia
        for instance_name in instance_names {
            let clean_instance = instance_name.trim();
            if clean_instance.is_empty() {
                continue;
            }

            let host_name = format!("{}.local.", clean_instance.to_lowercase().replace(' ', "-"));
            let properties: HashMap<String, String> = HashMap::new();

            // NOTA: Para la IP se pasa "" para dejar que mdns-sd autodetecte las interfaces locales
            let service_info = ServiceInfo::new(
                &self.service_type,
                clean_instance,
                &host_name,
                "", 
                self.port,
                properties,
            )?;

            let fullname = service_info.get_fullname().to_string();
            self.daemon.register(service_info)?;
            println!("📢 [mDNS - OK] Anunciado: {}", fullname);
            current.push(fullname);
        }

        Ok(())
    }

    pub async fn listen(&self, state: Arc<KachowState>) -> Result<(), Box<dyn std::error::Error>> {
        // Escuchar únicamente en el service_type común de la aplicación
        let receiver = self.daemon.browse(&self.service_type)?;
        let state_clone = state.clone();

        tokio::spawn(async move {
            while let Ok(event) = receiver.recv_async().await {
                // Obtener datos frescos del estado en cada evento recibido
                let contacts = state_clone.storage.get_all_contact_addresses().await;
                let valid_contacts: HashSet<String> = contacts
                    .into_iter()
                    .map(|c| c.secret_service_name)
                    .collect();

                let my_device_id = state_clone
                    .storage
                    .get_identity_device_id()
                    .await
                    .unwrap_or_default();
                
                let my_secret_name = state_clone
                    .storage
                    .get_identity_secret_service_name()
                    .await
                    .unwrap_or_default();

                match event {
                    ServiceEvent::ServiceResolved(info) => {
                        let fullname = info.get_fullname();
                        // Extraer el nombre de la instancia antes del primer punto
                        let instance_name = fullname.split('.').next().unwrap_or(fullname);
                        let lower_instance = instance_name.to_lowercase();
                        
                        // 🛑 DESCHARTAR SI ES PROPIO
                        if instance_name == my_secret_name {
                            continue;
                        }

                        // CASO 1: Es un anuncio público ("kachow-<device_id>")
                        if let Some(device_id) = lower_instance.strip_prefix("kachow-") {
                            if device_id == my_device_id || device_id.is_empty() {
                                continue; // Es el mismo dispositivo
                            }

                            println!(
                                "🔎 [PÚBLICO ENCONTRADO] Device ID: '{}' | Host: {}:{}",
                                device_id,
                                info.get_hostname(),
                                info.get_port()
                            );

                            println!("{:?}", info);

                            state_clone
                                .add_discovered_device(
                                    device_id.to_string(),
                                    format!("{}:{}", info.get_hostname(), info.get_port()),
                                )
                                .await;
                        } 
                        // CASO 2: Está registrado en la lista de contactos
                        else if valid_contacts.contains(instance_name) {
                            println!(
                                "🔎 [CONTACTO ENCONTRADO] Nombre: '{}' | Host: {}:{}",
                                instance_name,
                                info.get_hostname(),
                                info.get_port()
                            );
                            state_clone
                                .add_discovered_device(
                                    instance_name.to_string(),
                                    format!("{}:{}", info.get_hostname(), info.get_port()),
                                )
                                .await;
                        }
                    }
                    ServiceEvent::ServiceRemoved(_, fullname) => {
                        let instance_name = fullname.split('.').next().unwrap_or(&fullname);
                        let lower_instance = instance_name.to_lowercase();

                        let id_to_remove = if let Some(device_id) = lower_instance.strip_prefix("kachow-") {
                            device_id
                        } else {
                            instance_name
                        };

                        println!("❌ [DESCONECTADO] ID: {}", id_to_remove);
                        state_clone.remove_discovered_device(id_to_remove).await;
                    }
                    _ => {}
                }
            }
        });

        Ok(())
    }
}