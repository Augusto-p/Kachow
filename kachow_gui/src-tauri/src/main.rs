mod appHandle;
mod database;
mod win_ipc;
use kachow_ipc::ipc::client::IpcClient;

use crate::appHandle::set_app_handle;
use crate::database::Database;
use crate::win_ipc::{win_ipc_handle, IpcMessage};
use tauri::Listener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let storage = Database::open("kachow.db")?;
    let socket_path = storage.get_identity_ipc_socket_path().await;
    if socket_path.is_none() {
        return Err("No se Encotro la ruta del socket".into());
    }

    let ipc = IpcClient::new(&socket_path.unwrap());
    tauri::Builder::default()
    .plugin(tauri_plugin_dialog::init())
        .setup(move |app| {
            set_app_handle(app.handle().clone(), ipc);

            app.listen("IPC", move |event| {
                // 1. Deserializamos la carga útil (payload) dentro del evento síncrono
                if let Ok(msg) = serde_json::from_str::<IpcMessage>(event.payload().trim()) {
                    // 2. Generamos una tarea asíncrona (Task/Future)
                    // Si tienes tokio = { version = "1", features = ["full"] } en Cargo.toml
                    std::thread::spawn(move || {
                        tauri::async_runtime::block_on(win_ipc_handle(msg));
                    });
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error running tauri app");
    Ok(())
}
