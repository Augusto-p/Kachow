use kachow_ipc::ipc::{
    self,
    client::IpcClient,
    structs::{
        DeviceInfo, IpcRequest,
        IpcResponse::{self},
    },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{env, fs, path::PathBuf};

use crate::appHandle::get_api_ipc;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IpcMessage {
    pub name: String,
    pub data: serde_json::Value,
}
pub async fn win_ipc_handle(msg: IpcMessage) {
    match msg.name.as_str() {
        "Start" => {
            let api = get_api_ipc();
            match api.send_ipc(&IpcRequest::Info).await {
                Ok(IpcResponse::Info(mut device_info)) => {
                    if let Some(home) = dirs::home_dir() {
                        if let Some(home_str) = home.to_str() {
                            device_info.download_dir =
                                device_info.download_dir.replace(home_str, "~");
                        }
                    }

                    let _ = api.emit(IpcMessage {
                        name: "InfoMyDevice".to_string(),
                        data: serde_json::to_value(&device_info).unwrap(),
                    });
                }
                Ok(IpcResponse::Error(e)) => {
                    eprintln!("Error devuelto por el demonio: {}", e);
                }
                Ok(_) => {
                    println!("Respuesta inesperada del demonio.");
                }
                Err(e) => {
                    eprintln!("Error de comunicación IPC (conexión/socket): {}", e);
                }
            };
        }

        "SetNameDevice" => {
            let api = get_api_ipc();
            let name = msg.data["name"].as_str().unwrap_or("").trim().to_string();
            match api.send_ipc(&IpcRequest::SetNameDevice { name }).await {
                Ok(IpcResponse::Ok) => {}
                Ok(IpcResponse::Error(e)) => {
                    eprintln!("Error devuelto por el demonio: {}", e);
                }
                Ok(_) => {
                    println!("Respuesta inesperada del demonio.");
                }
                Err(e) => {
                    eprintln!("Error de comunicación IPC (conexión/socket): {}", e);
                }
            };
        }

        "SetImageDevice" => {
            let api = get_api_ipc();
            let image = msg.data["image"].as_str().unwrap_or("").trim().to_string();
            match api.send_ipc(&IpcRequest::SetImageDevice { image }).await {
                Ok(IpcResponse::Ok) => {}
                Ok(IpcResponse::Error(e)) => {
                    eprintln!("Error devuelto por el demonio: {}", e);
                }
                Ok(_) => {
                    println!("Respuesta inesperada del demonio.");
                }
                Err(e) => {
                    eprintln!("Error de comunicación IPC (conexión/socket): {}", e);
                }
            };
        }
        "SetDownloadFolder" => {
            let api = get_api_ipc();
            let path = msg.data["path"].as_str().unwrap_or("").trim().to_string();
            match api.send_ipc(&IpcRequest::SetDownloadFolder { path }).await {
                Ok(IpcResponse::Folder(path)) => {
                    let _ = api.emit(IpcMessage {
                        name: "LoadDownloadFolder".to_string(),
                        data: serde_json::from_str(&format!("{{\"download_dir\": \"{}\"}}", path))
                            .unwrap(),
                    });
                }
                Ok(IpcResponse::Error(e)) => {
                    eprintln!("Error devuelto por el demonio: {}", e);
                }
                Ok(_) => {
                    println!("Respuesta inesperada del demonio.");
                }
                Err(e) => {
                    eprintln!("Error de comunicación IPC (conexión/socket): {}", e);
                }
            };
        }

        "SetPrivateMode" => {
            let api = get_api_ipc();

            match api.send_ipc(&IpcRequest::SetModePrivate).await {
                Ok(IpcResponse::Ok) => {}
                Ok(IpcResponse::Error(e)) => {
                    eprintln!("Error devuelto por el demonio: {}", e);
                }
                Ok(_) => {
                    println!("Respuesta inesperada del demonio.");
                }
                Err(e) => {
                    eprintln!("Error de comunicación IPC (conexión/socket): {}", e);
                }
            };
        }
        "SetPublicMode" => {
            let api = get_api_ipc();
            match api.send_ipc(&IpcRequest::SetModePublic).await {
                Ok(IpcResponse::Ok) => match api.send_ipc(&IpcRequest::GetPairCode).await {
                    Ok(IpcResponse::PairCode(code, time)) => {
                        let _ = api.emit(IpcMessage {
                            name: "LoadPairCode".to_string(),
                            data: serde_json::from_str(&format!(
                                "{{\"pair_code\": \"{}\", \"time\": {}}}",
                                code.unwrap(),
                                time.unwrap()
                            ))
                            .unwrap(),
                        });
                    }
                    Ok(IpcResponse::Error(e)) => {
                        eprintln!("Error devuelto por el demonio: {}", e);
                    }
                    Ok(_) => {
                        println!("Respuesta inesperada del demonio.");
                    }
                    Err(e) => {
                        eprintln!("Error de comunicación IPC (conexión/socket): {}", e);
                    }
                },
                Ok(IpcResponse::Error(e)) => {
                    eprintln!("Error devuelto por el demonio: {}", e);
                }
                Ok(_) => {
                    println!("Respuesta inesperada del demonio.");
                }
                Err(e) => {
                    eprintln!("Error de comunicación IPC (conexión/socket): {}", e);
                }
            };
        }

        _ => {}
    }
}
