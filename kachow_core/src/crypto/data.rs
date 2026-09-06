use crypto_box::{aead::Aead, generate_nonce, Box as CryptoBox, Nonce, PublicKey, SecretKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

/// Estructura que contiene los datos encriptados necesarios para ser enviados.
#[derive(Serialize, Deserialize, Debug)]
pub struct EncryptedDataPayload {
    pub ciphertext: String,
    pub nonce: String,
    pub ephemeral_pk: String,
}

pub struct DataCrypt;
impl DataCrypt {
    /// Encripta un string (`data_str`) usando la clave pública (HEX) del dispositivo receptor.
    pub fn encrypt_data(
        data_str: &str,
        recipient_public_key_hex: &str,
    ) -> Result<EncryptedDataPayload, String> {
        // 1. Decodificar la clave pública desde Hex
        let pk_bytes = hex::decode(recipient_public_key_hex)
            .map_err(|e| format!("Error al decodificar Hex de clave pública: {}", e))?;

        let pk_array: [u8; 32] = pk_bytes
            .try_into()
            .map_err(|_| "La clave pública debe tener 32 bytes".to_string())?;

        let recipient_pk = PublicKey::from(pk_array);

        // 2. Generar clave efímera y nonce
        let mut rng = OsRng;
        let ephemeral_sk = SecretKey::generate(&mut rng);
        let nonce = generate_nonce(&mut rng);

        // 3. Cifrar con CryptoBox
        let cipher_box = CryptoBox::new(&recipient_pk, &ephemeral_sk);
        let ciphertext_bytes = cipher_box
            .encrypt(&nonce, data_str.as_bytes())
            .map_err(|e| format!("Error al encriptar: {}", e))?;

        // 4. Retornar DTO codificado en Hex
        Ok(EncryptedDataPayload {
            ciphertext: hex::encode(ciphertext_bytes),
            nonce: hex::encode(nonce),
            ephemeral_pk: hex::encode(ephemeral_sk.public_key().as_bytes()),
        })
    }

    /// Desencripta un `EncryptedPayload` usando la clave privada (HEX) del propio dispositivo.
    pub fn decrypt_data(
        payload: &EncryptedDataPayload,
        my_private_key_hex: &str,
    ) -> Result<String, String> {
        // 1. Decodificar la clave privada propia desde Hex
        let sk_bytes = hex::decode(my_private_key_hex)
            .map_err(|e| format!("Error al decodificar Hex de clave privada: {}", e))?;

        let sk_array: [u8; 32] = sk_bytes
            .try_into()
            .map_err(|_| "La clave privada debe tener 32 bytes".to_string())?;

        let my_sk = SecretKey::from(sk_array);

        // 2. Decodificar la clave pública efímera recibida
        let ephemeral_pk_bytes = hex::decode(&payload.ephemeral_pk)
            .map_err(|e| format!("Error al decodificar Hex de ephemeral_pk: {}", e))?;

        let ephemeral_pk_array: [u8; 32] = ephemeral_pk_bytes
            .try_into()
            .map_err(|_| "La clave efímera debe tener 32 bytes".to_string())?;

        let ephemeral_pk = PublicKey::from(ephemeral_pk_array);

        // 3. Decodificar Nonce y Ciphertext
        let nonce_bytes = hex::decode(&payload.nonce)
            .map_err(|e| format!("Error al decodificar Hex de nonce: {}", e))?;

        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext_bytes = hex::decode(&payload.ciphertext)
            .map_err(|e| format!("Error al decodificar Hex de ciphertext: {}", e))?;

        // 4. Descifrar con CryptoBox
        let cipher_box = CryptoBox::new(&ephemeral_pk, &my_sk);
        let decrypted_bytes = cipher_box
            .decrypt(nonce, ciphertext_bytes.as_slice())
            .map_err(|e| format!("Error al desencriptar payload: {}", e))?;

        String::from_utf8(decrypted_bytes)
            .map_err(|e| format!("Error al convertir bytes a String UTF-8: {}", e))
    }
}
