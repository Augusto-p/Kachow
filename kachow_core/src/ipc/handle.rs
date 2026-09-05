use std::{path::PathBuf, sync::Arc};

use kachow_ipc::ipc::structs::{Device, DeviceInfo, IpcRequest, IpcResponse};

use crate::{identity::keys::IdentityKeyPair, state::KachowState};

pub async fn handle_ipc_request(req: IpcRequest, state: Arc<KachowState>) -> IpcResponse {
    // pub async fn handle_ipc_request(req: IpcRequest) -> IpcResponse {

    match req {
        IpcRequest::Info => {
            let identity = state.storage.get_identity().await.unwrap();

            let info = DeviceInfo {
                device_name: identity.display_name,
                device_id: identity.device_id,
                tcp_port: identity.tcp_port,
                download_dir: identity.download_dir,
                device_image: "Soy una Imagen".to_string(),
            };
            return IpcResponse::Info(info);
        }

        IpcRequest::SetModePrivate => {
            state.change_mode(false).await;
            return IpcResponse::Ok;
        }

        IpcRequest::SetModePublic => {
            state.change_mode(true).await;
            return IpcResponse::Ok;
        }

        IpcRequest::GetPairCode => {
            if state.is_mode_active() {
                return IpcResponse::PairCode(state.pair_key.get_key());
            }
            return IpcResponse::Error("Private Mode".to_string());
        }

        IpcRequest::SendFiles {
            target_id,
            files_paths,
        } => {

            // hacer ping a target_id para verificar que está en línea antes de enviar los archivos
            println!(
                "Solicitud de envío de archivos recibida: target_id={}",
                target_id
            );

            let path_buf_files: Vec<PathBuf> = files_paths.into_iter().map(PathBuf::from).collect();
            for file in &path_buf_files {
                println!("Archivo a enviar: {}", file.display());
            }



            return IpcResponse::Ok;
        } 
        IpcRequest::Pair { target_id, pair_code } => {
            println!(
                "Solicitud de emparejamiento recibida: target_id={} con pair_code={}",
                target_id, pair_code
            );

            // Aquí se implementaría la lógica de emparejamiento real
            // Por ahora, simplemente devolvemos Ok para indicar que la solicitud fue recibida
            return IpcResponse::Ok;
        }
        IpcRequest::Discovered => {
            let discovered_devices = state.get_discovered_devices().await;
            let mut devices: Vec<Device> = Vec::new();
            let keys_option = state.storage.get_identity_secret_key().await;
            if keys_option.is_none() {
                return IpcResponse::Error("No se pudo obtener la clave secreta del dispositivo".to_string());
            }
            let keys = keys_option.unwrap();
            for device_id in &discovered_devices {
                if state.storage.has_contact(device_id).await {
                    let contact = state.storage.get_contact(device_id).await.unwrap();        
                    let code = keys.sign(&contact.public_key);
                    let url = format!("http://{}/info/{}/{}", state.get_device_value(device_id).await.unwrap(), state.storage.get_identity_device_id().await.unwrap_or_else(|| "".into()), &String::from_utf8_lossy(&code));
                    let response = reqwest::get(url).await;
                    match response {
                        Ok(resp) => {
                            if resp.status().is_success() {
                                match resp.json::<serde_json::Value>().await {
                                    Ok(device_info) => {
                                        devices.push(Device {
                                            device_name: device_info.get("device_name")
                                                .and_then(|info| info.as_str())
                                                .unwrap_or("Desconocido")
                                                .to_string(),
                                            device_id: device_info.get("device_id")
                                                .and_then(|info| info.as_str())
                                                .unwrap_or("Desconocido")
                                                .to_string(),
                                            device_image: device_info.get("device_image")
                                                .and_then(|info| info.as_str())
                                                .unwrap_or("Desconocido")
                                                .to_string(),
                                        });

                                }
                                    Err(_) => todo!(),
                                }
                            } else {
                                println!("Error en la respuesta HTTP: {}", resp.status());
                            }
                        }
                        Err(e) => {
                            println!("Error al realizar la solicitud HTTP: {}", e);
                        }
                    }
                }else{
                    let url = format!("http://{}/info/-/-", state.get_device_value(device_id).await.unwrap());
                    let response = reqwest::get(url).await;
                    match response {
                        Ok(resp) => {
                            if resp.status().is_success() {
                                match resp.json::<serde_json::Value>().await {
                                    Ok(response) => {
                                        let data = response.get("data");
                                        devices.push(Device {
                                            device_name: data
                                                .and_then(|info| info.get("device_name"))
                                                .and_then(serde_json::Value::as_str)
                                                .unwrap_or("Desconocido")
                                                .to_string(),
                                            device_id: data
                                                .and_then(|info| info.get("device_id"))
                                                .and_then(serde_json::Value::as_str)
                                                .unwrap_or("Desconocido")
                                                .to_string(),
                                            device_image: data
                                                .and_then(|info| info.get("device_image"))
                                                .and_then(serde_json::Value::as_str)
                                                .unwrap_or("Desconocido")
                                                .to_string(),
                                        });
                                    }
                                    Err(e) => {
                                        println!("Error al parsear la respuesta JSON: {}", e);
                                    }
                                }
                            } else {
                                println!("Error en la respuesta HTTP: {}", resp.status());
                            }
                        }
                        Err(e) => {
                            println!("Error al realizar la solicitud HTTP: {}", e);
                        }
                    }

                }
            }   
            IpcResponse::Discovered(devices)

        }
        // IpcRequest::GetVinculedKey => {
          //     // 1. Extraer la clave en un bloque cerrado
          //     let cached_key = {
          //         let guard = state.vinculed_key.read().await;
          //         guard
          //             .as_ref()
          //             .filter(|vk| vk.is_valid())
          //             .map(|vk| vk.key.clone())
          //     }; // <--- AQUÍ SE LIBERA EL READ LOCK

          //     // 2. Si existía y era válida, retornar de inmediato sin tocar el write lock
          //     if let Some(key) = cached_key {
          //         return IpcResponse::VinculedKey { key };
          //     }

          //     // 3. Adquirir el write lock (sin riesgo de deadlock)
          //     let mut guard = state.vinculed_key.write().await;
          //     // Re-verificar por si otro hilo la generó en la ventana de tiempo
          //     if let Some(ref vk) = *guard {
          //         if vk.is_valid() {
          //             return IpcResponse::VinculedKey {
          //                 key: vk.key.clone(),
          //             };
          //         }
          //     }

          //     // Generar y guardar la nueva clave
          //     let new_key = VinculedKey::new();
          //     let key_clone = new_key.key.clone();
          //     *guard = Some(new_key);

          //     IpcResponse::VinculedKey { key: key_clone }
          // }
          // IpcRequest::PingPeer { target_id } => {
          //     let peers = state.discovered_peers.read().await;

          //     if let Some(peer) = peers.get(&target_id) {
          //         let ip_str = peer.ip.to_string();
          //         let port = peer.port;
          //         let my_id = state.identity.device_id();
          //         drop(peers); // Liberar el read lock antes del await de red

          //         match P2pListener::ping_peer(&ip_str, port, &my_id).await {
          //             Ok(responder_id) => IpcResponse::PongReceived { responder_id },
          //             Err(e) => IpcResponse::Error(format!("Fallo el ping P2P: {}", e)),
          //         }
          //     } else {
          //         IpcResponse::Error(format!(
          //             "Peer '{}' no encontrado en la lista mDNS",
          //             target_id
          //         ))
          //     }
          // }
          // IpcRequest::GetStatus => {
          //     let active_count = state.active_transfers.read().await.len();
          //     IpcResponse::Status {
          //         running: true,
          //         active_transfers: active_count,
          //     }
          // }

          // IpcRequest::ListPeers => {
          //     let peers_map = state.discovered_peers.read().await;
          //     let peers: Vec<PeerInfo> = peers_map
          //         .values()
          //         .map(|p| PeerInfo {
          //             id: p.device_id.clone(),
          //             name: p.display_name.clone(),
          //             ip: p.ip.to_string(),
          //             port: p.port,
          //         })
          //         .collect();

          //     IpcResponse::Peers(peers)
          // }

          // IpcRequest::SendFile {
          //     target_id,
          //     file_path,
          // } => {
          //     let path = PathBuf::from(&file_path);
          //     if !path.exists() {
          //         return IpcResponse::Error(format!("El archivo no existe: {}", file_path));
          //     }

          //     let peer = {
          //         let peers = state.discovered_peers.read().await;
          //         match peers.get(&target_id) {
          //             Some(p) => p.clone(),
          //             None => {
          //                 return IpcResponse::Error("El dispositivo destino no está en línea".into())
          //             }
          //         }
          //     };

          //     let transfer_id = format!("tx_{}", uuid::Uuid::new_v4());
          //     let total_bytes = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

          //     state.active_transfers.write().await.insert(
          //         transfer_id.clone(),
          //         TransferProgress {
          //             transfer_id: transfer_id.clone(),
          //             bytes_sent: 0,
          //             total_bytes,
          //             speed_bytes_sec: 0,
          //         },
          //     );

          //     // Se puede hacer spawn del motor de envío P2P usando `peer.ip` y `peer.port`

          //     IpcResponse::TransferStarted { transfer_id }
          // }

          // IpcRequest::GetTransferStatus { transfer_id } => {
          //     let transfers = state.active_transfers.read().await;
          //     if let Some(progress) = transfers.get(&transfer_id) {
          //         IpcResponse::Progress(progress.clone())
          //     } else {
          //         IpcResponse::Error("Transferencia no encontrada".into())
          //     }
          // }

          // IpcRequest::CancelTransfer { transfer_id } => {
          //     let mut transfers = state.active_transfers.write().await;
          //     if transfers.remove(&transfer_id).is_some() {
          //         IpcResponse::Ok
          //     } else {
          //         IpcResponse::Error("No se pudo cancelar: la transferencia no existe".into())
          //     }
          // }
    }
}
