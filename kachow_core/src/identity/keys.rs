use std::vec;

use base64::{engine::general_purpose, Engine as _};
use crypto_box::{
    aead::AeadCore, aead::AeadMutInPlace, Nonce, PublicKey as BoxPublicKey, SalsaBox,
    SecretKey as BoxSecretKey,
};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256, Sha512};

#[derive(Debug, Clone)]
pub struct IdentityKeyPair {
    pub signing_key: SigningKey,
    pub verifying_key: VerifyingKey,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EncryptedDataPayload {
    pub ciphertext_b64: String,
    pub nonce_b64: String,
    pub ephemeral_pk_b64: String,
}

impl IdentityKeyPair {
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

    pub fn from_slice(secret_bytes: &[u8]) -> Option<Self> {
        let bytes_32: [u8; 32] = secret_bytes.try_into().ok()?;
        let signing_key = SigningKey::from_bytes(&bytes_32);
        let verifying_key = signing_key.verifying_key();

        Some(Self {
            signing_key,
            verifying_key,
        })
    }

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

    pub fn to_bytes(&self) -> [u8; 32] {
        self.signing_key.to_bytes()
    }

    pub fn device_id(&self) -> String {
        derive_device_id(&self.verifying_key)
    }

    pub fn valid(public_key_bytes: Vec<u8>, mensaje: &[u8], firma: &[u8]) -> bool {
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

    // =========================================================================
    // MÉTODOS DE CIFRADO / DESCIFRADO ADAPTADOS
    // =========================================================================

    /// Encripta un string usando la clave pública (Base64) del receptor.
    /// Encripta un string usando la clave pública (Base64 o HEX) del receptor.
pub fn encrypt_for_recipient(
    data_str: &str,
    recipient_public_key_str: &str,
) -> Result<EncryptedDataPayload, String> {
    // Limpiar posibles espacios en blanco, saltos de línea o comillas
    let clean_pk_str = recipient_public_key_str
        .trim()
        .trim_matches('"')
        .trim_matches('\'');

    // 1. Decodificar la clave pública soportando HEX (64 chars) o Base64 (44 chars / estándar)
    let pk_bytes = if clean_pk_str.len() == 64 && clean_pk_str.chars().all(|c| c.is_ascii_hexdigit()) {
        // Formato Hexadecimal
        (0..clean_pk_str.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&clean_pk_str[i..i + 2], 16))
            .collect::<Result<Vec<u8>, _>>()
            .map_err(|e| format!("Error al decodificar HEX en clave pública: {}", e))?
    } else {
        // Formato Base64
        general_purpose::STANDARD
            .decode(clean_pk_str)
            .map_err(|e| format!("Error Base64 en clave pública: {}", e))?
    };

    // 2. Validar que la longitud resultante sea exactamente de 32 bytes (256 bits)
    let pk_bytes_32: [u8; 32] = pk_bytes
        .try_into()
        .map_err(|v: Vec<u8>| {
            format!(
                "La clave pública debe ser de 32 bytes (se recibieron {} bytes desde: '{}')",
                v.len(),
                clean_pk_str
            )
        })?;

    // 3. Reconstruir la clave pública Ed25519
    // 3. Reconstruir la clave pública Ed25519
    let recipient_ed_pk = VerifyingKey::from_bytes(&pk_bytes_32)
        .map_err(|e| format!("Clave pública Ed25519 inválida: {}", e))?;

    // 4. Convertir VerifyingKey (Ed25519) -> EdwardsPoint -> MontgomeryPoint (X25519) -> BoxPublicKey
    let edward_point = recipient_ed_pk.to_edwards();
    let montgomery_point = edward_point.to_montgomery();
    let recipient_x25519_pk = BoxPublicKey::from(montgomery_point.0);
    // 5. Generar un par de claves efímero X25519 y un nonce
    let mut rng = OsRng;
    let ephemeral_sk = BoxSecretKey::generate(&mut rng);
    let nonce = SalsaBox::generate_nonce(&mut rng);

    // 6. Cifrar con SalsaBox
    use crypto_box::aead::Aead;
    let cipher_box = SalsaBox::new(&recipient_x25519_pk, &ephemeral_sk);
    let ciphertext = cipher_box
        .encrypt(&nonce, data_str.as_bytes())
        .map_err(|e| format!("Error en el cifrado: {}", e))?;

    // 7. Empaquetar y devolver la estructura serializable en Base64
    Ok(EncryptedDataPayload {
        ciphertext_b64: general_purpose::STANDARD.encode(ciphertext),
        nonce_b64: general_purpose::STANDARD.encode(nonce),
        ephemeral_pk_b64: general_purpose::STANDARD.encode(ephemeral_sk.public_key().as_bytes()),
    })
}
    /// Descifra el payload recibido usando la propia clave privada del par de claves.
    pub fn decrypt(&self, payload: &EncryptedDataPayload) -> Result<String, String> {
        // 1. Derivar la BoxSecretKey X25519 aplicando SHA-512 + Clamping a la SigningKey Ed25519
        let mut hasher = Sha512::new();
        hasher.update(self.signing_key.to_bytes());
        let hash = hasher.finalize();

        let mut x25519_sk_bytes = [0u8; 32];
        x25519_sk_bytes.copy_from_slice(&hash[..32]);

        x25519_sk_bytes[0] &= 248;
        x25519_sk_bytes[31] &= 127;
        x25519_sk_bytes[31] |= 64;

        let my_x25519_sk = BoxSecretKey::from(x25519_sk_bytes);

        // 2. Decodificar la clave efímera recibida
        let ephemeral_pk_bytes = general_purpose::STANDARD
            .decode(&payload.ephemeral_pk_b64)
            .map_err(|e| format!("Error Base64 en ephemeral_pk: {}", e))?;

        let ephemeral_pk_32: [u8; 32] = ephemeral_pk_bytes
            .try_into()
            .map_err(|_| "La clave efímera debe ser de 32 bytes".to_string())?;

        let ephemeral_x25519_pk = BoxPublicKey::from(ephemeral_pk_32);

        // 3. Decodificar nonce y ciphertext
        let nonce_bytes = general_purpose::STANDARD
            .decode(&payload.nonce_b64)
            .map_err(|e| format!("Error Base64 en nonce: {}", e))?;

        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = general_purpose::STANDARD
            .decode(&payload.ciphertext_b64)
            .map_err(|e| format!("Error Base64 en ciphertext: {}", e))?;

        // 4. Descifrar con SalsaBox
        use crypto_box::aead::Aead;
        let cipher_box = SalsaBox::new(&ephemeral_x25519_pk, &my_x25519_sk);

        let decrypted_bytes = cipher_box
            .decrypt(nonce, ciphertext.as_slice())
            .map_err(|e| format!("Error al descifrar payload: {}", e))?;

        String::from_utf8(decrypted_bytes)
            .map_err(|e| format!("Error UTF-8 en datos descifrados: {}", e))
    }
}

pub fn derive_device_id(public_key: &VerifyingKey) -> String {
    let mut hasher = Sha256::new();
    hasher.update(public_key.as_bytes());
    let result = hasher.finalize();
    hex::encode(&result[..8])
}

mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
}
