use crate::{
    crypto::{files::CryptoManager, pair::{EncryptedPayload, PairCrypt}}, database::contacts::Contact, identity::keys::{EncryptedDataPayload, IdentityKeyPair}, mdns::MdnsManager, state::KachowState,
};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use dirs::download_dir;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use axum::{
    body::Bytes,
};
use serde_json::{json, Value};

pub struct WEB;

#[derive(Serialize)]
pub struct ApiResponse<T>
where
    T: Serialize,
{
    pub success: bool,
    pub code: u16,
    pub message: String,
    pub data: Option<T>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KachowPair {
    pub name: String,
    pub public_key: Vec<u8>,
    pub secret_service_name: String,
    pub device_id: String,
    pub device_name: String,
    pub device_image: String,
}

#[derive(Serialize)]
pub struct KachowDeviceInfo {
    pub device_id: String,
    pub device_name: String,
    pub device_image: String,
}

impl WEB {
    pub async fn info(
        Path((device_id, code)): Path<(String, String)>,
        State(state): State<Arc<KachowState>>,
    ) -> (StatusCode, Json<ApiResponse<KachowDeviceInfo>>) {
        println!("Solicitud de información para el dispositivo: {device_id}, con código: {code}");
        let storage = state.storage.clone();
        if state.is_mode_active() {
            let device_id_me = storage.get_identity_device_id().await.unwrap();
            let device_name = storage.get_identity_display_name().await.unwrap();
            let device_image = storage.get_identity_device_image().await.unwrap();

            return (
                StatusCode::OK,
                Json(ApiResponse {
                    success: true,
                    code: 200,
                    message: "Info Kachow".to_string(),
                    data: Some(KachowDeviceInfo {
                        device_id: device_id_me,
                        device_name,
                        device_image,
                    }),
                }),
            );
        } else {
            if let Some(identity_key) = storage.get_identity_secret_key().await {
                let your_public_key = storage
                    .get_contact_public_key(&device_id)
                    .await
                    .unwrap_or_else(|| Vec::new());
                if IdentityKeyPair::valid(
                    your_public_key,
                    identity_key.verifying_key.as_bytes(),
                    code.as_bytes(),
                ) {
                    let device_id_me = storage.get_identity_device_id().await.unwrap();
                    let device_name = storage.get_identity_display_name().await.unwrap();
                    let device_image = storage.get_identity_device_image().await.unwrap();
                    return (
                        StatusCode::OK,
                        Json(ApiResponse {
                            success: true,
                            code: 200,
                            message: "Info Kachow".to_string(),
                            data: Some(KachowDeviceInfo {
                                device_id: device_id_me,
                                device_name,
                                device_image,
                            }),
                        }),
                    );
                }
            }
        }

        (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<KachowDeviceInfo> {
                success: false,
                code: 403,
                message: "No Gateway".to_string(),
                data: None,
            }),
        )
    }

    pub async fn pair(
        Path(device_id): Path<String>,
        ws: WebSocketUpgrade,
        State(state): State<Arc<KachowState>>,
    ) -> Response {
        // 👈 Cambiado de Box<dyn IntoResponse> a Response
        ws.on_upgrade(move |socket| Self::handle_socket(socket, device_id, state))
            .into_response() // Convierte a Response<Body>
    }
    async fn handle_socket(mut socket: WebSocket, _device_id: String, state: Arc<KachowState>) {
        // 1. Enviar "Ready" al cliente inmediatamente
        if socket.send(Message::Text("Ready".into())).await.is_err() {
            return;
        }

        // 2. Bucle de recepción de mensajes
        while let Some(Ok(msg)) = socket.recv().await {
            if let Message::Text(payload_json) = msg {
                // Paso A: Recibir Kachow-Alpha cifrado con PairCode
                if let Ok(pair_payload) = serde_json::from_str::<EncryptedPayload>(&payload_json) {
                    let payload_data = match PairCrypt::desencriptar(
                        &pair_payload,
                        &state.pair_key.get_key().unwrap(),
                    ) {
                        Ok(data) => data,
                        Err(_) => break,
                    };

                    let json_payload: serde_json::Value = match serde_json::from_str(&payload_data)
                    {
                        Ok(json) => json,
                        Err(_) => continue,
                    };

                    // Normalización de claves en minúsculas
                    if json_payload["Name"] == "Kachow-Alpha"
                        && json_payload["public_key"].is_string()
                    {
                        let public_key = json_payload["public_key"].as_str().unwrap_or_default();

                        let secret_key = state.storage.get_identity_secret_key().await.unwrap();
                        let data_beta = KachowPair {
                            name: "Kachow-Beta".to_string(),
                            public_key: secret_key.verifying_key.as_bytes().to_vec(),
                            secret_service_name: MdnsManager::normalize_service_type(
                                &state
                                    .storage
                                    .get_identity_secret_service_name()
                                    .await
                                    .unwrap_or_default(),
                            ),
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

                        let data_str = serde_json::to_string(&data_beta).unwrap();
                        let encrypted_response =
                            IdentityKeyPair::encrypt_for_recipient(&data_str, public_key).unwrap();
                        let json_out = serde_json::to_string(&encrypted_response).unwrap();

                        let _ = socket.send(Message::Text(json_out.into())).await;
                    }
                }

                // Paso B: Recibir Kachow-Gama cifrado asimétricamente
                if let Ok(pair_response) =
                    serde_json::from_str::<EncryptedDataPayload>(&payload_json)
                {
                    let secret_key = state.storage.get_identity_secret_key().await.unwrap();
                    if let Ok(data_original) = secret_key.decrypt(&pair_response) {
                        if let Ok(json_payload) =
                            serde_json::from_str::<serde_json::Value>(&data_original)
                        {
                            if json_payload["name"] == "Kachow-Gama" {
                                let _ = state
                                    .storage
                                    .set_contact(&Contact {
                                        device_id: json_payload["device_id"]
                                            .as_str()
                                            .unwrap_or_default()
                                            .to_string(),
                                        secret_service_name: MdnsManager::normalize_service_type(
                                            &json_payload["secret_service_name"]
                                                .as_str()
                                                .unwrap_or_default()
                                                .to_string(),
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
                                        public_key: json_payload["public_key"]
                                            .as_array()
                                            .map(|arr| {
                                                arr.iter()
                                                    .filter_map(|v| v.as_u64().map(|n| n as u8))
                                                    .collect()
                                            })
                                            .unwrap_or_default(),
                                    })
                                    .await;
                                let _ = socket.send(Message::Text("Save".into())).await;
                                break; // Cierra la conexión después de guardar
                            }
                        }
                    }
                }
            }
        }
    }

    pub async fn receive(
    Path(device_id): Path<String>,
    State(state): State<Arc<KachowState>>,
    body_bytes: Bytes, // Obtiene los bytes crudos del cuerpo de la petición
) -> (StatusCode, Json<ApiResponse<Value>>) {
    // `body` es un buffer de bytes (&[u8] / Bytes)
    println!("Device ID: {}, Tamaño en bytes: {}", device_id, body_bytes.len());
    let my_key = state.storage.get_identity_secret_key().await.unwrap();
    let dir = state.storage.get_identity_download_dir().await.unwrap();
    let download_dir = std::path::Path::new(&dir);;
    if let Some(public_key) = state.storage.get_contact_public_key(&device_id).await {
        return match CryptoManager::decrypt_unpack_and_verify(
            &body_bytes,
            &my_key,
            &public_key,
            &download_dir,
        ) {
            Ok(_) => (
                StatusCode::OK,
                Json(ApiResponse {
                    success: true,
                    data: json!({ "message": "Archivos descifrados y guardados correctamente" }).into(),
                    code: 200,
                    message: "Archivos descifrados y guardados correctamente".to_string(),
                }),
            ),
            Err(err) => {
                eprintln!("Error al procesar recepción: {:?}", err);
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse {
                        success: false,
                        data: json!({ "error": err.to_string() }).into(),
                        code: 400,
                        message: err.to_string(),
                    }),
                );
            }
        };
    }


    let response = ApiResponse {
        success: true,
        data: json!({
            "message": "Bytes recibidos correctamente",
            "byte_count": body_bytes.len()
        }).into(),
        code: 200,
        message: "Bytes recibidos correctamente".to_string(),
    };

    (StatusCode::OK, Json(response))
}
}
