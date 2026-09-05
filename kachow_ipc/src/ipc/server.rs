use std::path::Path;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
// #[cfg(unix)]
use tokio::net::UnixListener;

use crate::ipc::structs::{IpcRequest, IpcResponse};





pub struct IpcServer;

impl IpcServer {
    // #[cfg(unix)]
    pub async fn listen<F, Fut>(socket_path: &str, handler: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: Fn(IpcRequest) -> Fut + Send + Sync + 'static + Clone,
        Fut: std::future::Future<Output = IpcResponse> + Send,
    {
        if Path::new(socket_path).exists() {
            let _ = std::fs::remove_file(socket_path);
        }

        let listener = UnixListener::bind(socket_path)?;

        println!("Servidor IPC escuchando en Unix Socket: {}", socket_path);

        loop {
            match listener.accept().await {
                Ok((mut stream, _)) => {
                    let handler_clone = handler.clone();

                    tokio::spawn(async move {
                        // ─────────────────────────────────────
                        // Leer longitud del mensaje: 4 bytes
                        // ─────────────────────────────────────
                        let mut len_bytes = [0u8; 4];

                        if stream.read_exact(&mut len_bytes).await.is_err() {
                            return;
                        }

                        let len = u32::from_be_bytes(len_bytes) as usize;

                        // ─────────────────────────────────────
                        // Leer payload
                        // ─────────────────────────────────────
                        let mut buf = vec![0u8; len];

                        if stream.read_exact(&mut buf).await.is_err() {
                            return;
                        }

                        // ─────────────────────────────────────
                        // Deserializar request
                        // ─────────────────────────────────────
                        let request = match serde_json::from_slice::<IpcRequest>(&buf) {
                            Ok(request) => request,
                            Err(e) => {
                                println!("Error deserializando request IPC: {}", e);
                                return;
                            }
                        };

                        // ─────────────────────────────────────
                        // Ejecutar handler
                        // ─────────────────────────────────────
                        let response = handler_clone(request).await;

                        // ─────────────────────────────────────
                        // Serializar response
                        // ─────────────────────────────────────
                        let resp_bytes = match serde_json::to_vec(&response) {
                            Ok(bytes) => bytes,
                            Err(e) => {
                                println!("Error serializando respuesta IPC: {}", e);
                                return;
                            }
                        };

                        // ─────────────────────────────────────
                        // Enviar longitud
                        // ─────────────────────────────────────
                        let resp_len = (resp_bytes.len() as u32).to_be_bytes();

                        if stream.write_all(&resp_len).await.is_err() {
                            return;
                        }

                        // ─────────────────────────────────────
                        // Enviar payload
                        // ─────────────────────────────────────
                        if stream.write_all(&resp_bytes).await.is_err() {
                            return;
                        }

                        let _ = stream.flush().await;
                    });
                }

                Err(e) => {
                    println!("Error aceptando cliente IPC: {}", e);
                }
            }
        }
    }
}


