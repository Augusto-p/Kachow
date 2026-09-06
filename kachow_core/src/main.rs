mod crypto;
mod database;
mod identity;
mod ipc;
mod mdns;
mod state;
mod utils;
mod web;
use std::{sync::Arc, time::Duration};

use axum::{Router, routing::{get, post}};
use kachow_ipc::ipc::server::IpcServer;
use tokio::time::sleep;

use crate::{
    database::{identity::LocalIdentity, Database},
    ipc::handle::handle_ipc_request,
    mdns::MdnsManager,
    state::KachowState,
    web::WEB,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let storage = Arc::new(Database::open("kachow.db")?);

    load_identity_or_default(storage.clone()).await;

    let state = Arc::new(KachowState::new(storage.clone()));

    let tcp_port = storage.get_identity_tcp_port().await.unwrap_or(9533);

    let ipc_socket_path = storage
        .get_identity_ipc_socket_path()
        .await
        .unwrap_or("/run/user/1000/kachow.sock".to_string());

    // -------------------------
    // WEB
    // -------------------------

    let web_state = state.clone();

    let app = Router::new()
        .route("/pair/{device_id}", get(WEB::pair))
        .route("/info/{device_id}/{code}", get(WEB::info))
        .with_state(web_state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", tcp_port)).await?;

    println!("Kachow Web escuchando en 0.0.0.0:{}", tcp_port);

    let web_task = tokio::spawn(async move {
        if let Err(err) = axum::serve(listener, app).await {
            eprintln!("Error servidor Web: {}", err);
        }
    });

    // -------------------------
    // IPC
    // -------------------------

    let ipc_state = state.clone();

    let ipc_task = tokio::spawn(async move {
        if let Err(err) = IpcServer::listen(&ipc_socket_path, move |req| {
            let state = ipc_state.clone();

            async move { handle_ipc_request(req, state).await }
        })
        .await
        {
            eprintln!("Error servidor IPC: {}", err);
        }
    });

    // -------------------------
    // mDNS
    // -------------------------

    let mdns_state = state.clone();
    // 1. Escuchar los servicios dinámicos configurados en `state`

    // Asume el tipo de servicio mDNS para HTTP en la red local
    // Modifica la inicialización eliminando '.local.'
    // Debe terminar explícitamente en .local.
    let mdns_manager = MdnsManager::new("_http._tcp.local.", tcp_port)?;
    mdns_manager.listen(state.clone()).await?;
    println!("🔎 Listener mDNS iniciado.");
    let mdns_task = tokio::spawn(async move {
        loop {
            // Evaluamos la condición del estado
            // (Ajusta la llamada a la función/campo real de `KachowState` si varía)
            let is_mode_active = mdns_state.is_mode_active();

            let instance_name = if is_mode_active {
                &("kachow-".to_string()
                    + &mdns_state
                        .storage
                        .get_identity_device_id()
                        .await
                        .unwrap_or_else(|| "".into()))
            } else {
                &mdns_state
                    .storage
                    .get_identity_secret_service_name()
                    .await
                    .unwrap_or_else(|| "".into())
            };

            if let Err(err) = mdns_manager.announce(instance_name) {
                eprintln!("Error al anunciar mDNS: {}", err);
            }

            // Esperar 30 segundos antes de la siguiente verificación/re-anuncio
            sleep(Duration::from_secs(5)).await;
        }
    });

    // Esperar las tres tareas concurrentes
    tokio::try_join!(web_task, ipc_task, mdns_task)?;
    // Esperar ambos
    /* tokio::try_join!(web_task, ipc_task)?;/*  */ */

    Ok(())
}

async fn load_identity_or_default(db: Arc<Database>) {
    if db.get_identity().await.is_none() {
        let identity_def = LocalIdentity::default();

        if !db.set_identity(&identity_def).await {
            eprintln!("Error: No se pudo guardar la identidad por defecto.");
            std::process::exit(1);
        }
    }
}
