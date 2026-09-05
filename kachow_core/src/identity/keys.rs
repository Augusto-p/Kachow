use std::vec;

use base64::{engine::general_purpose, Engine as _};
use ed25519_dalek::{Signature, Verifier};
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
pub struct IdentityKeyPair {
    pub signing_key: SigningKey,
    pub verifying_key: VerifyingKey,
}

impl IdentityKeyPair {
    /// Genera un nuevo par de claves Ed25519 de forma infalible.
    pub fn generate() -> Self {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();

        Self {
            signing_key,
            verifying_key,
        }
    }

    pub fn public_key_to_string(&self) -> String {
        general_purpose::STANDARD.encode(self.verifying_key.to_bytes())
    }

    /// Intenta reconstruir el par de claves desde un slice de bytes.
    /// Retorna `None` si la longitud del slice no es exactamente de 32 bytes.
    pub fn from_slice(secret_bytes: &[u8]) -> Option<Self> {
        let bytes_32: [u8; 32] = secret_bytes.try_into().ok()?;
        let signing_key = SigningKey::from_bytes(&bytes_32);
        let verifying_key = signing_key.verifying_key();

        Some(Self {
            signing_key,
            verifying_key,
        })
    }

    /// Reconstruye el par de claves desde el array exacto de 32 bytes de la clave privada.
    pub fn from_bytes(secret_bytes: &[u8; 32]) -> Self {
        let signing_key = SigningKey::from_bytes(secret_bytes);
        let verifying_key = signing_key.verifying_key();

        Self {
            signing_key,
            verifying_key,
        }
    }

    pub fn sign(&self, nonce: &[u8]) -> Vec<u8> {
        let signature = self.signing_key.sign(nonce);
        signature.to_vec()
    }

    /// Exporta los bytes de la clave privada para almacenamiento seguro.
    pub fn to_bytes(&self) -> [u8; 32] {
        self.signing_key.to_bytes()
    }

    /// Deriva el identificador único e inmutable del dispositivo (16 caracteres HEX).
    pub fn device_id(&self) -> String {
        derive_device_id(&self.verifying_key)
    }

    pub fn valid(
    public_key_bytes: Vec<u8>,
    mensaje: &[u8],
    firma: &[u8],
) -> bool {
    

    let public_key_bytes: [u8; 32] = match public_key_bytes.try_into() {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };

    let public_key = match VerifyingKey::from_bytes(&public_key_bytes) {
        Ok(key) => key,
        Err(_) => return false,
    };

    let firma: [u8; 64] = match firma.try_into() {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };

    let firma = Signature::from_bytes(&firma);

    public_key.verify(mensaje, &firma).is_ok()
}
}

/// Función pura que calcula SHA256(PublicKey)[..8] codificado en HEX.
pub fn derive_device_id(public_key: &VerifyingKey) -> String {
    let mut hasher = Sha256::new();
    hasher.update(public_key.as_bytes());
    let result = hasher.finalize();
    hex::encode(&result[..8])
}

// Utilidad auxiliar interna para codificación HEX sin dependencias pesadas
mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
}
