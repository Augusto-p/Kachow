use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use std::collections::HashMap;
use std::fmt::format;
use std::net::IpAddr;
use std::os::unix::raw::dev_t;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::state::KachowState;

// Función para obtener la IP local principal de la máquina
fn get_local_ip() -> Option<String> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().ok().map(|addr| addr.ip().to_string())
}
pub struct MdnsManager {
    daemon: ServiceDaemon,
    // Guardamos el fullname del servicio registrado actualmente para poder de-registrarlo
    current_fullname: Arc<Mutex<Option<String>>>,
    service_type: String,
    port: u16,
}

impl MdnsManager {
    /// Crea una nueva instancia del gestor mDNS
    pub fn new(service_type: &str, port: u16) -> Result<Self, Box<dyn std::error::Error>> {
        let daemon = ServiceDaemon::new()?;
        Ok(Self {
            daemon,
            current_fullname: Arc::new(Mutex::new(None)),
            service_type: service_type.to_string(),
            port,
        })
    }

    /// Anuncia (o re-anuncia) el servicio con un nuevo nombre de instancia
    pub fn announce(&self, instance_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut current = self.current_fullname.lock().unwrap();

        if let Some(ref old_fullname) = *current {
            let _ = self.daemon.unregister(old_fullname);
        }

        let host_name = format!("{}.local.", instance_name.to_lowercase().replace(' ', "-"));
        let properties = HashMap::new();

        // Obtener la IP explícita
        let my_ip = get_local_ip().unwrap_or_default();

        let service_info = ServiceInfo::new(
            &self.service_type,
            instance_name,
            &host_name,
            &my_ip, // <-- Pasar la IP real en lugar de ""
            self.port,
            properties,
        )?;

        let fullname = service_info.get_fullname().to_string();
        self.daemon.register(service_info)?;

        *current = Some(fullname);
        Ok(())
    }

    pub async fn listen(&self, state: Arc<KachowState>) -> Result<(), Box<dyn std::error::Error>> {
        let contacts = state.storage.get_all_contact_addresses().await;

        let mut service_types: Vec<String> = contacts
            .into_iter()
            .map(|c| c.secret_service_name)
            .collect();

        if !service_types.contains(&self.service_type) {
            service_types.push(self.service_type.clone());
        }

        for service_type in service_types {
            let receiver = self.daemon.browse(&service_type)?;
            let service_type_owned = service_type.clone();
            let state_clone = state.clone();
            let device_id_me = state.storage.get_identity_device_id().await.unwrap_or_else(|| "".into());
            tokio::spawn(async move {
                while let Ok(event) = receiver.recv_async().await {
                    match event {
                        ServiceEvent::ServiceResolved(info) => {
                            let fullname = info.get_fullname();
                            
                            // 1. Extraer solo la parte del nombre de la instancia (antes del primer punto)
                            // Ejemplo: "kachow-xyz123._http._tcp.local." -> "kachow-xyz123"
                            let instance_name = fullname.split('.').next().unwrap_or(fullname);

                            let lower_instance = instance_name.to_lowercase();

                            // 2. Comprobar si empieza por "kachow-"
                            if let Some(device_id) = lower_instance.strip_prefix("kachow-") {
                                if device_id == device_id_me {
                                    continue; // Ignorar nuestro propio anuncio
                                }
                                if !device_id.is_empty() {
                                    println!(
                                        "🔎 [Encontrado - {}] Device ID: '{}' | Host: {}:{}",
                                        service_type_owned,
                                        device_id,
                                        info.get_hostname(),
                                        info.get_port()
                                    );

                                    // Guardamos únicamente el Device ID
                                    state_clone
                                        .add_discovered_device(
                                            device_id.to_string(),
                                            format!("{}:{}", info.get_hostname(), info.get_port()),
                                        )
                                        .await;
                                }
                            } else if service_type_owned != "_http._tcp.local." {
                                
                                // Si es un servicio de la lista explícita pero no empieza por kachow-
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

                            // Extraer el id correspondiente para removerlo del estado
                            let id_to_remove =
                                if let Some(device_id) = lower_instance.strip_prefix("kachow-") {
                                    device_id
                                } else {
                                    instance_name
                                };

                            println!(
                                "❌ [Desconectado - {}] Device ID: {}",
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
