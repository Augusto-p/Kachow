use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::Serialize;

use crate::{database::identity, identity::keys::IdentityKeyPair, state::KachowState};

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
        println!("Solicitud de información para el dispositivo:, con código: ",);
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
                let your_public_key = storage.get_contact_public_key(&device_id).await.unwrap_or_else(|| Vec::new());
                if IdentityKeyPair::valid(
                    your_public_key,
                    identity_key.verifying_key.as_bytes(),
                    &code.as_bytes(),
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
}
