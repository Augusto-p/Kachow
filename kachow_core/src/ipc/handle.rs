use std::{path::PathBuf, sync::Arc};

use crate::{
    crypto::pair::{EncryptedPayload, PairCrypt},
    database::contacts::Contact,
    identity::keys::{EncryptedDataPayload, IdentityKeyPair},
    mdns::MdnsManager,
    state::KachowState,
    web::KachowPair,
};
use futures_util::{SinkExt, StreamExt};
use kachow_ipc::ipc::structs::{Device, DeviceInfo, IpcRequest, IpcResponse};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

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
        IpcRequest::Pair {
            target_id,
            pair_code,
        } => {
            println!(
                "Solicitud de emparejamiento recibida: target_id={}, pair_code={}",
                target_id, pair_code
            );
            let identity = match state.storage.get_identity().await {
                Some(id) => id,
                None => {
                    return IpcResponse::Error("No se pudo obtener la identidad local".to_string())
                }
            };

            let mdns_name = state
                .get_device_value(&target_id)
                .await
                .unwrap_or_else(|| "".to_string());

            let key = match state.storage.get_identity_secret_key().await {
                Some(k) => k,
                None => {
                    return IpcResponse::Error("No se pudo obtener la clave secreta".to_string())
                }
            };

            let pair_data: serde_json::Value = serde_json::json!({
                "public_key": key.public_key_to_string(),
                "Name": "Kachow-Alpha",
            });

            let public_key_crypto =
                match PairCrypt::encriptar(&serde_json::to_string(&pair_data).unwrap(), &pair_code)
                {
                    Ok(crypto) => crypto,
                    Err(e) => return IpcResponse::Error(format!("Error al encriptar: {e}")),
                };

            let url = format!("ws://{}/pair/{}", mdns_name, identity.device_id);
            println!("Conectando a {url}...");

            let (ws_stream, _) = match connect_async(&url).await {
                Ok(conn) => conn,
                Err(e) => return IpcResponse::Error(format!("Error al conectar WebSocket: {e}")),
            };

            println!("Conexión WebSocket establecida.");
            let (mut write, mut read) = ws_stream.split();

            while let Some(msg_result) = read.next().await {
                let msg = match msg_result {
                    Ok(m) => m,
                    Err(e) => {
                        return IpcResponse::Error(format!("Error en el stream WebSocket: {e}"))
                    }
                };

                if let Message::Text(text) = msg {
                    println!("Mensaje recibido del servidor: {text}");

                    if text == "Save" {
                        return IpcResponse::Ok;
                    } else if text == "Ready" {
                        println!("Enviando public_key_crypto al servidor...");
                        let payload = match serde_json::to_string(&public_key_crypto) {
                            Ok(json) => json,
                            Err(e) => {
                                return IpcResponse::Error(format!("Error al serializar JSON: {e}"))
                            }
                        };

                        if let Err(e) = write.send(Message::Text(payload)).await {
                            return IpcResponse::Error(format!("Error al enviar el mensaje: {e}"));
                        }
                    } else {
                        // Intentar procesar como respuesta cifrada de emparejamiento (Kachow-Beta)
                        if let Ok(pair_response) =
                            serde_json::from_str::<EncryptedDataPayload>(&text)
                        {
                            let secret_key = state.storage.get_identity_secret_key().await.unwrap();
                            let data_original = secret_key.decrypt(&pair_response).unwrap();
                            let json_payload: serde_json::Value =
                                serde_json::from_str(&data_original)
                                    .expect("Error al deserializar el payload de emparejamiento");

                            println!("Respuesta de emparejamiento recibida: {:?}", json_payload);
                            if json_payload["name"] == "Kachow-Beta" {
                                let public_key: Vec<u8> = json_payload["public_key"]
                                    .as_array()
                                    .map(|arr| {
                                        arr.iter()
                                            .filter_map(|v| v.as_u64().map(|n| n as u8))
                                            .collect()
                                    })
                                    .unwrap_or_default();
                                

                                
                                let _ = state
                                    .storage
                                    .set_contact(&Contact {
                                        device_id: json_payload["device_id"]
                                            .as_str()
                                            .unwrap_or_default()
                                            .to_string(),
                                        secret_service_name: MdnsManager::normalize_service_type(
                                            json_payload["secret_service_name"].as_str().unwrap(),
                                        ),
                                        device_image: json_payload["device_image"]
                                            .as_array()
                                            .unwrap_or(&vec![])
                                            .iter()
                                            .map(|v| v.as_u64().unwrap_or(0) as u8)
                                            .collect::<Vec<u8>>(),
                                        display_name: json_payload["device_name"]
                                            .as_str()
                                            .unwrap_or_default()
                                            .to_string(),
                                        public_key: public_key.clone(),
                                    })
                                    .await;

                                let data_me = KachowPair {
                                    name: "Kachow-Gama".to_string(),
                                    public_key: secret_key.verifying_key.as_bytes().to_vec(),
                                    secret_service_name: state
                                        .storage
                                        .get_identity_secret_service_name()
                                        .await
                                        .unwrap_or_else(|| "Unknown".to_string()),
                                    device_id: state
                                        .storage
                                        .get_identity_device_id()
                                        .await
                                        .unwrap_or_else(|| "Unknown".to_string()),
                                    device_name: state
                                        .storage
                                        .get_identity_display_name()
                                        .await
                                        .unwrap_or_else(|| "Unknown".to_string()),
                                    device_image: state
                                        .storage
                                        .get_identity_device_image()
                                        .await
                                        .unwrap_or_default(),
                                };

                                let data_str = serde_json::to_string(&data_me).unwrap();
                                let encrypted_response =
                                    IdentityKeyPair::encrypt_for_recipient(&data_str, &hex::encode(public_key))
                                        .map_err(|err| IpcResponse::Error(err.to_string()));
                                let json_out = serde_json::to_string(&encrypted_response).unwrap();
                                println!("Enviando confirmación Gama al servidor...");
                                if let Err(e) = write.send(Message::Text(json_out)).await {
                                    return IpcResponse::Error(format!(
                                        "Error al enviar confirmación Gama: {e}"
                                    ));
                                }
                                return IpcResponse::Ok;
                            } else {
                                return IpcResponse::Error(
                                    "Respuesta de emparejamiento inválida".to_string(),
                                );
                            }
                        }
                    }
                } else if let Message::Close(_) = msg {
                    println!("El servidor cerró la conexión.");
                    return IpcResponse::Error(
                        "El servidor cerró la conexión inesperadamente".to_string(),
                    );
                }
            }

            IpcResponse::Ok
        }
        IpcRequest::Discovered => {
            let discovered_devices = state.get_discovered_devices().await;
            let mut devices: Vec<Device> = Vec::new();
            let keys_option = state.storage.get_identity_secret_key().await;
            if keys_option.is_none() {
                return IpcResponse::Error(
                    "No se pudo obtener la clave secreta del dispositivo".to_string(),
                );
            }
            let keys = keys_option.unwrap();
            for device_id in &discovered_devices {
                if state.storage.has_contact(device_id).await {
                    let contact = state.storage.get_contact(device_id).await.unwrap();
                    let code = keys.sign(&contact.public_key);
                    let url = format!(
                        "http://{}/info/{}/{}",
                        state.get_device_value(device_id).await.unwrap(),
                        state
                            .storage
                            .get_identity_device_id()
                            .await
                            .unwrap_or_else(|| "".into()),
                        &String::from_utf8_lossy(&code)
                    );
                    let response = reqwest::get(url).await;
                    match response {
                        Ok(resp) => {
                            if resp.status().is_success() {
                                match resp.json::<serde_json::Value>().await {
                                    Ok(device_info) => {
                                        devices.push(Device {
                                            device_name: device_info
                                                .get("device_name")
                                                .and_then(|info| info.as_str())
                                                .unwrap_or("Desconocido")
                                                .to_string(),
                                            device_id: device_info
                                                .get("device_id")
                                                .and_then(|info| info.as_str())
                                                .unwrap_or("Desconocido")
                                                .to_string(),
                                            device_image: device_info
                                                .get("device_image")
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
                } else {
                    let url = format!(
                        "http://{}/info/-/-",
                        state.get_device_value(device_id).await.unwrap()
                    );
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
        } // IpcRequest::GetVinculedKey => {
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
