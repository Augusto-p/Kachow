use base64::engine::general_purpose;
use base64::Engine as _;


use std::io::{Error, ErrorKind, Result};
use std::path::{Path, PathBuf};
use tar::Builder as TarBuilder;
use zstd::stream::write::Encoder as ZstdEncoder;

use crate::identity::keys::{EncryptedDataPayload, IdentityKeyPair};
use serde::{Deserialize, Serialize};
use std::{
    fs::self
};
use tar::Archive as TarArchive;
use zstd::stream::read::Decoder as ZstdDecoder;
#[derive(Serialize, Deserialize, Debug)]
pub struct SignedEncryptedPackage {
    pub encrypted_payload: EncryptedDataPayload, // Datos cifrados para el receptor
    pub signature_b64: String,                   // Firma digital generada con TU clave privada
    pub sender_device_id: String,                // Identificador de quien envía
}

pub struct CryptoManager;

impl CryptoManager {
    pub fn pack_compress_encrypt_and_sign(
        files: &[PathBuf],
        recipient_public_key_bytes: &[u8],
        my_identity: &IdentityKeyPair, // Tu clave privada para firmar
    ) -> Result<Vec<u8>> {
        // 1. Validar la clave pública del destinatario (32 bytes)
        if recipient_public_key_bytes.len() != 32 {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "La clave pública del destinatario debe ser de 32 bytes",
            ));
        }
        let recipient_pk_b64 = general_purpose::STANDARD.encode(recipient_public_key_bytes);

        // 2. Empacar en TAR y comprimir con Zstd
        let mut compressed_payload = Vec::new();
        {
            let zstd_encoder = ZstdEncoder::new(&mut compressed_payload, 3)?;
            let mut tar_builder = TarBuilder::new(zstd_encoder);
            for file_path in files {
                if file_path.is_file() {
                    // Asigna la ruta dentro del archivo TAR usando solo el nombre del archivo
                    if let Some(file_name) = file_path.file_name() {
                        tar_builder.append_path_with_name(file_path, file_name)?;
                    }
                }
            }

            let zstd_stream = tar_builder.into_inner()?;
            zstd_stream.finish()?;
        }

        // 3. Cifrar con la clave pública del destinatario
        let compressed_b64 = general_purpose::STANDARD.encode(&compressed_payload);
        let encrypted_payload =
            IdentityKeyPair::encrypt_for_recipient(&compressed_b64, &recipient_pk_b64)
                .map_err(|e| Error::new(ErrorKind::Other, format!("Error en el cifrado: {e}")))?;

        // 4. Firmar con TU clave privada
        let signature_bytes = my_identity.sign(encrypted_payload.ciphertext_b64.as_bytes());
        let signature_b64 = general_purpose::STANDARD.encode(signature_bytes);

        // 5. Empaquetar todo con tu device_id
        let package = SignedEncryptedPackage {
            encrypted_payload,
            signature_b64,
            sender_device_id: my_identity.device_id(),
        };

        // 6. Serializar a bytes para enviar por Axum
        serde_json::to_vec(&package).map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("Error al serializar paquete: {e}"),
            )
        })
    }

pub fn decrypt_unpack_and_verify(
    package_bytes: &[u8],
    my_identity: &IdentityKeyPair,
    sender_public_key_bytes: &[u8],
    output_dir: &Path,
) -> Result<()> {
    // 1. Deserializar el paquete firmado recibido
    let pkg: SignedEncryptedPackage = serde_json::from_slice(package_bytes)
        .map_err(|e| Error::new(ErrorKind::InvalidData, format!("JSON inválido: {e}")))?;

    // 2. Verificar la firma usando la clave pública del remitente (Ed25519)
    let signature_bytes = general_purpose::STANDARD
        .decode(&pkg.signature_b64)
        .map_err(|e| Error::new(ErrorKind::InvalidData, format!("Firma Base64 inválida: {e}")))?;

    let is_valid = IdentityKeyPair::valid(
        sender_public_key_bytes.to_vec(),
        pkg.encrypted_payload.ciphertext_b64.as_bytes(),
        &signature_bytes,
    );

    if !is_valid {
        return Err(Error::new(
            ErrorKind::PermissionDenied,
            "Firma digital inválida: el mensaje fue alterado o no proviene del remitente esperado",
        ));
    }

    // 3. Descifrar el payload usando nuestra clave privada (SalsaBox / X25519)
    let compressed_b64 = my_identity
        .decrypt(&pkg.encrypted_payload)
        .map_err(|e| Error::new(ErrorKind::Other, format!("Error al descifrar payload: {e}")))?;

    let compressed_bytes = general_purpose::STANDARD
        .decode(&compressed_b64)
        .map_err(|e| Error::new(ErrorKind::InvalidData, format!("Base64 comprimido inválido: {e}")))?;

    // 4. Crear el directorio de salida si no existe
    if !output_dir.exists() {
        fs::create_dir_all(output_dir)?;
    }

    // 5. Descomprimir Zstd y desempaquetar TAR directamente hacia la carpeta dada
    let zstd_decoder = ZstdDecoder::new(&compressed_bytes[..])?;
    let mut tar_archive = TarArchive::new(zstd_decoder);

    tar_archive.unpack(output_dir)?;

    println!("Archivos desempaquetados con éxito en: {}", output_dir.display());
    Ok(())
}
}
