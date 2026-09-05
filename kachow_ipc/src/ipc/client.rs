use std::{path::Path, error::Error};
use tokio::net::UnixStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::ipc::structs::{IpcRequest, IpcResponse};


pub struct IpcClient {
    socket_path: String,
}

impl IpcClient {
    pub fn new(socket_path: &str) -> Self {
        Self {
            socket_path: socket_path.to_string(),
        }
    }

    pub async fn send(&self, request: &IpcRequest) -> Result<IpcResponse, Box<dyn Error + Send + Sync>> {
        if !Path::new(&self.socket_path).exists() {
            return Err("El demonio de kachow no está ejecutándose (socket IPC no encontrado)".into());
        }

        let mut stream = UnixStream::connect(&self.socket_path).await?;

        // 1. Serializar Request
        let req_bytes = serde_json::to_vec(request)?;
        let len = (req_bytes.len() as u32).to_be_bytes();

        // 2. Enviar Tamaño + Payload
        stream.write_all(&len).await?;
        stream.write_all(&req_bytes).await?;
        stream.flush().await?;

        // 3. Leer tamaño de la respuesta
        let mut resp_len_bytes = [0u8; 4];
        stream.read_exact(&mut resp_len_bytes).await?;
        let resp_len = u32::from_be_bytes(resp_len_bytes) as usize;

        // 4. Leer payload
        let mut resp_buf = vec![0u8; resp_len];
        stream.read_exact(&mut resp_buf).await?;

        // 5. Deserializar respuesta
        let response: IpcResponse = serde_json::from_slice(&resp_buf)?;

        Ok(response)
    }
}


