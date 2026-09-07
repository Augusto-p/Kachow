mod database;
use std::{fs, path::PathBuf};

use clap::{Parser, Subcommand};
use kachow_ipc::ipc::{
    self,
    client::IpcClient,
    structs::{
        IpcRequest,
        IpcResponse::{self},
    },
};

use crate::database::Database;

#[derive(Parser)]
#[command(name = "kachow")]
#[command(about = "CLI para transferencia rápida de archivos P2P en Rust", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Info,
    SetModePublic,
    SetModePrivate,
    GetPairCode,
    Pair {
        #[arg(short, long)]
        target: String,

        #[arg(short, long)]
        pair_code: String,
    },
    Send {
        #[arg(short, long)]
        target: String,

        /// Lista de archivos a enviar
        #[arg(short, long, num_args = 1..)]
        files: Vec<PathBuf>,
    },
    Discovered,
    SetNameDevice {
        #[arg(short, long)]
        name: String,
    },
    SetImageDevice {
        #[arg(short, long)]
        image: String,
    },
    SetDownloadFolder {
        #[arg(short, long)]
        path: String,
    },
    // Ping {
    //     #[arg(short, long)]
    //     target: String,
    // },
    // /// Muestra el estado actual del demonio kachow y transferencias activas
    // Status,
    // GetVinculedKey,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let storage = Database::open("kachow.db")?;
    let socket_path = storage.get_identity_ipc_socket_path().await;
    if socket_path.is_none() {
        return Err("No se Encotro la ruta del socket".into());
    }

    let ipc = IpcClient::new(&socket_path.unwrap());

    match cli.command {
        Commands::Info => match ipc.send(&IpcRequest::Info).await {
            Ok(IpcResponse::Info(device_info)) => {
                println!("{}", device_info);
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
        Commands::GetPairCode => match ipc.send(&IpcRequest::GetPairCode).await {
            Ok(IpcResponse::PairCode(code, time)) => {
                println!("Pair Code: {}, ({})", code.unwrap(), time.unwrap());
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
        Commands::SetModePrivate => match ipc.send(&IpcRequest::SetModePrivate).await {
            Ok(IpcResponse::Ok) => {
                println!("Private Mode OK");
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

        Commands::SetModePublic => match ipc.send(&IpcRequest::SetModePublic).await {
            Ok(IpcResponse::Ok) => {
                println!("Public Mode OK");
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
        Commands::Pair { target, pair_code } => match ipc.send(&IpcRequest::Pair { target_id: target, pair_code }).await 
        {
            Ok(IpcResponse::Ok) => {
                println!("Pair OK");
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
        }
        Commands::Send { target, files } => {
            // Implement the send command logic here
            println!("Enviando archivos a {}", target);
            let absolute_files: Vec<PathBuf> = files
                .into_iter()
                .filter_map(|p| fs::canonicalize(p).ok())
                .collect();
            
            let string_files: Vec<String> = absolute_files
                .into_iter()
                .map(|p| p.to_string_lossy().into_owned())
                .collect();
            
            match ipc.send(&IpcRequest::SendFiles { target_id: target, files_paths: string_files }).await {
            Ok(IpcResponse::Ok) => {
                println!("Send Files OK");
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
        }
        }
        Commands::Discovered => {
            // Implement the discovered command logic here
            println!("Discovered devices:");
            match ipc.send(&IpcRequest::Discovered).await {
                Ok(IpcResponse::Discovered(devices)) => {
                    for device in devices {
                        println!("{}", device);
                    }
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
            }
        }
    
        Commands::SetNameDevice { name } => {
            let name_clone = name.clone();
            match ipc.send(&IpcRequest::SetNameDevice { name }).await {
            Ok(IpcResponse::Ok) => {
                println!("Setted {} with device name", name_clone);
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
        }},
        Commands::SetImageDevice { image } => {

            match ipc.send(&IpcRequest::SetImageDevice { image }).await {
            Ok(IpcResponse::Ok) => {
                println!("Setted Image Device");
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
        }},
                Commands::SetDownloadFolder { path } => {

            match ipc.send(&IpcRequest::SetDownloadFolder { path }).await {
            Ok(IpcResponse::Folder(path)) => {
                println!("Setted Folder {}", path);
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
        }},

    }

    Ok(())
}
