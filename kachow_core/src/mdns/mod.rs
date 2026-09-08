use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use std::collections::HashSet;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::state::KachowState;

fn get_local_ip() -> Option<String> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().ok().map(|addr| addr.ip().to_string())
}

pub struct MdnsManager {
    daemon: ServiceDaemon,
    current_fullnames: Arc<Mutex<Vec<String>>>,
    service_type: String,
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

    /// Anuncia simultáneamente todos los nombres indicados en `instance_names`
    pub fn announce_names(&self, instance_names: &[String]) -> Result<(), Box<dyn std::error::Error>> {
        let mut current = self.current_fullnames.lock().unwrap();

        // Limpiar anuncios anteriores
        for old_fullname in current.drain(..) {
            let _ = self.daemon.unregister(&old_fullname);
        }

        let my_ip = get_local_ip().ok_or("No se pudo obtener la IP local principal")?;

        for instance_name in instance_names {
            if instance_name.trim().is_empty() {
                continue;
            }

            let host_name = format!("{}.local.", instance_name.to_lowercase().replace(' ', "-"));
            let properties = HashMap::new();

            let service_info = ServiceInfo::new(
                &self.service_type,
                instance_name,
                &host_name,
                &my_ip,
                self.port,
                properties,
            )?;

            let fullname = service_info.get_fullname().to_string();
            self.daemon.register(service_info)?;
            println!("📢 [mDNS] Servicio registrado: {}", fullname);
            current.push(fullname);
        }

        Ok(())
    }

    pub fn normalize_service_type(service_type: &str) -> String {
        let trimmed = service_type.trim_matches('.');
        if trimmed.ends_with("_tcp.local") || trimmed.ends_with("_udp.local") {
            format!("{}.", trimmed)
        } else {
            format!("_{}._tcp.local.", trimmed)
        }
    }

    pub async fn listen(&self, state: Arc<KachowState>) -> Result<(), Box<dyn std::error::Error>> {
        let contacts = state.storage.get_all_contact_addresses().await;

        // Conjunto de secret_service_name de nuestros contactos para búsquedas rápidas
        let valid_contacts: HashSet<String> = contacts
            .iter()
            .map(|c| c.secret_service_name.clone())
            .collect();

        // Tipos de servicio que exploraremos
        let mut service_types: Vec<String> = contacts
            .into_iter()
            .map(|c| Self::normalize_service_type(&c.secret_service_name))
            .collect();

        if !service_types.contains(&self.service_type) {
            service_types.push(self.service_type.clone());
        }

        for service_type in service_types {
            let receiver = self.daemon.browse(&service_type)?;
            let service_type_owned = service_type.clone();
            let state_clone = state.clone();
            let valid_contacts_clone = valid_contacts.clone();

            // Identificadores propios para ignorar transmisiones del propio dispositivo
            let my_device_id = state
                .storage
                .get_identity_device_id()
                .await
                .unwrap_or_default();
            let my_secret_service_name = state
                .storage
                .get_identity_secret_service_name()
                .await
                .unwrap_or_default();

            tokio::spawn(async move {
                while let Ok(event) = receiver.recv_async().await {
                    match event {
                        ServiceEvent::ServiceResolved(info) => {
                            let fullname = info.get_fullname();
                            let instance_name = fullname.split('.').next().unwrap_or(fullname);
                            let lower_instance = instance_name.to_lowercase();

                            // 🛑 FILTRO 1: Ignorar si es nuestro propio dispositivo
                            if instance_name == my_secret_service_name {
                                continue;
                            }
                            if let Some(device_id) = lower_instance.strip_prefix("kachow-") {
                                if device_id == my_device_id {
                                    continue;
                                }

                                // ✅ CASO A: Dispositivo público con prefijo "kachow-"
                                if !device_id.is_empty() {
                                    println!(
                                        "🔎 [Encontrado público - {}] Device ID: '{}' | Host: {}:{}",
                                        service_type_owned,
                                        device_id,
                                        info.get_hostname(),
                                        info.get_port()
                                    );

                                    state_clone
                                        .add_discovered_device(
                                            device_id.to_string(),
                                            format!("{}:{}", info.get_hostname(), info.get_port()),
                                        )
                                        .await;
                                }
                            } 
                            // ✅ CASO B: Dispositivo en la lista de contactos
                            else if valid_contacts_clone.contains(instance_name) {
                                println!(
                                    "🔎 [Encontrado contacto - {}] Contacto: '{}' | Host: {}:{}",
                                    service_type_owned,
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

                            let id_to_remove =
                                if let Some(device_id) = lower_instance.strip_prefix("kachow-") {
                                    device_id
                                } else {
                                    instance_name
                                };

                            println!(
                                "❌ [Desconectado - {}] ID: {}",
                                service_type_owned, id_to_remove
                            );
                            state_clone.remove_discovered_device(id_to_remove).await;
                        }
                        _ => {}
                    }
                }
            });
        }
        Ok(())
    }
}