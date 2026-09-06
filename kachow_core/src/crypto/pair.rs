
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use argon2::Argon2;
use rand::RngCore;
use serde::{Deserialize, Serialize};

/// Representa los datos cifrados junto con los metadatos necesarios para desencriptar.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedPayload {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; 12],
    pub salt: [u8; 16],
}

pub struct PairCrypt;

impl PairCrypt {
    /// Deriva una clave simétrica de 32 bytes usando Argon2id.
    fn derivar_clave(clave: &str, salt: &[u8; 16]) -> Result<[u8; 32], String> {
        let mut clave_derivada = [0u8; 32];
        Argon2::default()
            .hash_password_into(clave.as_bytes(), salt, &mut clave_derivada)
            .map_err(|e| format!("Error en derivación de clave (KDF): {}", e))?;
        Ok(clave_derivada)
    }

    /// Encripta un texto en plano utilizando una clave corta.
    pub fn encriptar(texto: &str, clave: &str) -> Result<EncryptedPayload, String> {
        let mut salt = [0u8; 16];
        let mut nonce_bytes = [0u8; 12];

        rand::thread_rng().fill_bytes(&mut salt);
        rand::thread_rng().fill_bytes(&mut nonce_bytes);

        let clave_32_bytes = Self::derivar_clave(clave, &salt)?;
        let cipher = Aes256Gcm::new_from_slice(&clave_32_bytes)
            .map_err(|e| format!("Error al instanciar cipher: {}", e))?;

        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher
            .encrypt(nonce, texto.as_bytes())
            .map_err(|e| format!("Error al encriptar: {}", e))?;

        Ok(EncryptedPayload {
            ciphertext,
            nonce: nonce_bytes,
            salt,
        })
    }

    /// Desencripta una estructura `EncryptedPayload` devolviendo el texto original.
    pub fn desencriptar(payload: &EncryptedPayload, clave: &str) -> Result<String, String> {
        let clave_32_bytes = Self::derivar_clave(clave, &payload.salt)?;
        let cipher = Aes256Gcm::new_from_slice(&clave_32_bytes)
            .map_err(|e| format!("Error al instanciar cipher: {}", e))?;

        let nonce = Nonce::from_slice(&payload.nonce);
        let bytes_desencriptados = cipher
            .decrypt(nonce, payload.ciphertext.as_slice())
            .map_err(|_| "Clave incorrecta o datos alterados".to_string())?;

        String::from_utf8(bytes_desencriptados)
            .map_err(|e| format!("Error al convertir a UTF-8: {}", e))
    }
}
