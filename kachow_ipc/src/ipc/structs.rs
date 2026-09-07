use std::{fmt, string};

use serde::{Deserialize, Serialize};



#[derive(Serialize, Deserialize, Debug)]
pub enum IpcRequest {
    Info,
    SetModePublic,
    SetModePrivate,
    GetPairCode,
    SendFiles {
        target_id: String,
        files_paths: Vec<String>,
    },
    Pair {
        target_id: String,
        pair_code: String,
    },
    Discovered,
    SetNameDevice{
        name: String
    },
    SetImageDevice{
        image: String
    },
    SetDownloadFolder{
        path: String
    },

    // ValidPairCode,
    // GetStatus,
    // ListPeers,
    

}

#[derive(Serialize, Deserialize, Debug)]
pub enum IpcResponse {
    Ok,
    Error(String),
    Info(DeviceInfo),
    PairCode(Option<String>, Option<u64>),
    Discovered(Vec<Device>),
    Folder(String),
    // Status {
    //     running: bool,
    //     active_transfers: usize,
    // },
    
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Device {
    pub device_id: String,
    pub device_name: String,
    pub device_image: String,
}
impl fmt::Display for Device {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Device: {} ({}) |  Image: {}",
            self.device_name, self.device_id, self.device_image
        )
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeviceInfo {
    pub device_id: String,
    pub device_name: String,
    pub tcp_port: u16,
    pub download_dir: String,
    pub device_image: String,
}

impl fmt::Display for DeviceInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Device: {} ({}) | Port: {} | Dir: {} | Image: {}",
            self.device_name, self.device_id, self.tcp_port, self.download_dir, self.device_image
        )
    }
}
