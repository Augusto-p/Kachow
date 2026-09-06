use aes_gcm::{
    aead::{AeadInPlace, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::RngCore;
use rsa::{sha2::Sha256, Oaep, RsaPublicKey};
use std::fs::File;
use std::io::{Error, ErrorKind, Result, Write};
use std::path::PathBuf;
use tar::Builder as TarBuilder;
use zstd::stream::write::Encoder as ZstdEncoder;

pub struct CryptoManager;

impl CryptoManager {
    /// Empaqueta, comprime y cifra una lista de archivos directamente en memoria (`Vec<u8>`).
    pub fn pack_compress_and_encrypt(
        files: &[PathBuf],
        public_key: &RsaPublicKey,
    ) -> Result<Vec<u8>> {
        // 1. Generar la clave simétrica AES-256 (32 bytes) y el Nonce (12 bytes)
        let mut aes_key_bytes = [0u8; 32];
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut aes_key_bytes);
        rand::thread_rng().fill_bytes(&mut nonce_bytes);

        // 2. Cifrar la clave simétrica con la clave pública RSA del destinatario
        let padding = Oaep::new::<Sha256>();
        let encrypted_aes_key = public_key
            .encrypt(&mut rand::thread_rng(), padding, &aes_key_bytes)
            .map_err(|e| Error::new(ErrorKind::Other, format!("Error cifrando clave RSA: {e}")))?;

        // 3. Crear el buffer de salida final que se mantendrá en RAM
        let mut final_payload = Vec::new();

        // 4. Escribir Encabezado: [Tamaño Clave RSA (2 bytes)] + [Clave RSA Cifrada] + [Nonce (12 bytes)]
        let key_len_bytes = (encrypted_aes_key.len() as u16).to_be_bytes();
        final_payload.write_all(&key_len_bytes)?;
        final_payload.write_all(&encrypted_aes_key)?;
        final_payload.write_all(&nonce_bytes)?;

        // 5. Buffer temporal para canalizar TAR -> ZSTD en memoria
        let mut unencrypted_buffer = Vec::new();

        {
            // Tubería en memoria: TarBuilder escribe en ZstdEncoder, que escribe en un Vec<u8>
            let zstd_encoder = ZstdEncoder::new(&mut unencrypted_buffer, 3)?; // Nivel 3 de compresión
            let mut tar_builder = TarBuilder::new(zstd_encoder);

            for file_path in files {
                if file_path.is_file() {
                    let mut file = File::open(file_path)?;
                    let filename = file_path
                        .file_name()
                        .ok_or_else(|| Error::new(ErrorKind::InvalidInput, "Nombre de archivo inválido"))?;

                    // Añadir archivo al TAR
                    tar_builder.append_file(filename, &mut file)?;
                }
            }

            // Finalizar empaquetado TAR y compresión ZSTD
            let zstd_stream = tar_builder.into_inner()?;
            zstd_stream.finish()?;
        }

        // 6. Cifrar los datos comprimidos en memoria usando AES-256-GCM
        let cipher = Aes256Gcm::new_from_slice(&aes_key_bytes)
            .map_err(|e| Error::new(ErrorKind::Other, format!("Error iniciando AES: {e}")))?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        cipher
            .encrypt_in_place(nonce, b"", &mut unencrypted_buffer)
            .map_err(|e| Error::new(ErrorKind::Other, format!("Error cifrando datos con AES: {e}")))?;

        // 7. Adjuntar los datos cifrados al payload final
        final_payload.write_all(&unencrypted_buffer)?;

        Ok(final_payload)
    }
}