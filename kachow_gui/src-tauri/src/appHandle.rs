use kachow_ipc::ipc::client::IpcClient;
use kachow_ipc::ipc::structs::{IpcRequest, IpcResponse};
use tauri::AppHandle;
use once_cell::sync::OnceCell;
use serde::Serialize;
use serde::Deserialize;
use tauri::Emitter;
use std::sync::{Arc, Mutex};

use crate::win_ipc::IpcMessage;

static API_IPC: OnceCell<APP> = OnceCell::new();

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmitPayload {
    name: String,
    data: serde_json::Value,
}
impl From<IpcMessage> for EmitPayload {
    fn from(msg: IpcMessage) -> Self {
        EmitPayload {
            name: msg.name,
            data: msg.data,
        }
    }
}




#[derive(Clone)]
pub struct APP {
    app_handle: AppHandle,
    ipc: Arc<Mutex<IpcClient>>,
}

impl APP {
    pub fn new(handle: AppHandle,  ipc: IpcClient) -> Self {
        Self {
            app_handle: handle,
            ipc: Arc::new(Mutex::new(ipc))
        }
    }
    pub fn emit<T: EmitArg>(&self, msg: T) -> tauri::Result<()> {
            msg.emit(&self.app_handle)
        }
    
    pub fn get_handle(&self)->AppHandle{
        self.app_handle.clone()
    }

    pub fn ipc(&self) -> Arc<Mutex<IpcClient>> {
        self.ipc.clone()
    }
       pub fn with_ipc<F, R>(&self, f: F) -> R 
    where 
        F: FnOnce(&mut IpcClient) -> R 
    {
        let mut ipc = self.ipc.lock().expect("Failed to lock config");
        f(&mut ipc)
    }
   pub async fn send_ipc(&self, req: &IpcRequest) -> Result<IpcResponse, String> {
        // 1. Quitar el .await de lock()
        // 2. Extraer el guard con expect() o un match
        let mut ipc = self.ipc.lock().expect("Failed to lock IPC");
        
        // El .await va únicamente en la llamada asíncrona
        ipc.send(req).await.map_err(|e| e.to_string())
    }
   
}
pub trait EmitArg {
    fn emit(self, app: &AppHandle) -> tauri::Result<()>;
    
}
impl EmitArg for EmitPayload {
    fn emit(self, app: &AppHandle) -> tauri::Result<()> {
        app.emit("ipc", self)
    }

    
}

impl EmitArg for IpcMessage {
    fn emit(self, app: &AppHandle) -> tauri::Result<()> {
        let payload = EmitPayload::from(self);
        app.emit("ipc", payload)
    }

}

pub fn set_app_handle(handle: AppHandle, ipc: IpcClient) {
    let app = APP::new(handle, ipc);
    if let Err(_) = API_IPC.set(app) {
        println!("API_IPC ya estaba inicializado");
    }
}

pub fn get_api_ipc() -> &'static APP {
    API_IPC.get().expect("AppHandle no inicializado")
}