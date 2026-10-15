use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::{aead::Aead, Key, KeyInit, XChaCha20Poly1305, XNonce};
use rand::Rng;
use serde::{Deserialize, Serialize};

const FORMAT_VERSION: u8 = 1;
const KEY_SIZE: usize = 32;
const SALT_SIZE: usize = 16;
const NONCE_SIZE: usize = 24;
const KDF_MEMORY_KIB: u32 = 19_456;
const KDF_ITERATIONS: u32 = 2;
const KDF_PARALLELISM: u32 = 1;

const DATA_KEY_AAD: &[u8] = b"keepass-drive-vault:data-key:v1";
const PAYLOAD_AAD: &[u8] = b"keepass-drive-vault:payload:v1";

#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("authentication failed")]
    AuthenticationFailed,
    #[error("unsupported container version")]
    UnsupportedVersion,
    #[error("invalid container format")]
    InvalidContainer,
    #[error("key derivation failed")]
    KeyDerivationFailed,
    #[error("container serialization failed")]
    SerializationFailed,
}

#[derive(Debug, Clone, Copy)]
pub struct KdfParams {
    pub memory_kib: u32,
    pub iterations: u32,
    pub parallelism: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContainerMetadata {
    pub version: u8,
}

#[derive(Debug, Serialize, Deserialize)]
struct EncryptedContainer {
    version: u8,
    memory_kib: u32,
    iterations: u32,
    parallelism: u32,
    salt: Vec<u8>,
    wrap_nonce: Vec<u8>,
    wrapped_key: Vec<u8>,
    data_nonce: Vec<u8>,
    ciphertext: Vec<u8>,
}

pub fn derive_key(
    password: &[u8],
    salt: &[u8],
    params: KdfParams,
) -> Result<[u8; KEY_SIZE], CryptoError> {
    let argon_params = Params::new(
        params.memory_kib,
        params.iterations,
        params.parallelism,
        Some(KEY_SIZE),
    )
    .map_err(|_| CryptoError::KeyDerivationFailed)?;
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, argon_params);
    let mut key = [0u8; KEY_SIZE];
    argon
        .hash_password_into(password, salt, &mut key)
        .map_err(|_| CryptoError::KeyDerivationFailed)?;
    Ok(key)
}

pub fn encrypt_container(password: &[u8], payload: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let params = KdfParams {
        memory_kib: KDF_MEMORY_KIB,
        iterations: KDF_ITERATIONS,
        parallelism: KDF_PARALLELISM,
    };
    let mut rng = rand::rng();
    let mut salt = [0u8; SALT_SIZE];
    let mut wrap_nonce = [0u8; NONCE_SIZE];
    let mut data_nonce = [0u8; NONCE_SIZE];
    let mut data_key = [0u8; KEY_SIZE];
    rng.fill(&mut salt);
    rng.fill(&mut wrap_nonce);
    rng.fill(&mut data_nonce);
    rng.fill(&mut data_key);

    let derived_key = derive_key(password, &salt, params)?;
    let wrapping_cipher = XChaCha20Poly1305::new(Key::from_slice(&derived_key));
    let wrapped_key = wrapping_cipher
        .encrypt(
            XNonce::from_slice(&wrap_nonce),
            chacha20poly1305::aead::Payload {
                msg: &data_key,
                aad: DATA_KEY_AAD,
            },
        )
        .map_err(|_| CryptoError::AuthenticationFailed)?;

    let data_cipher = XChaCha20Poly1305::new(Key::from_slice(&data_key));
    let ciphertext = data_cipher
        .encrypt(
            XNonce::from_slice(&data_nonce),
            chacha20poly1305::aead::Payload {
                msg: payload,
                aad: PAYLOAD_AAD,
            },
        )
        .map_err(|_| CryptoError::AuthenticationFailed)?;

    serde_json::to_vec(&EncryptedContainer {
        version: FORMAT_VERSION,
        memory_kib: params.memory_kib,
        iterations: params.iterations,
        parallelism: params.parallelism,
        salt: salt.to_vec(),
        wrap_nonce: wrap_nonce.to_vec(),
        wrapped_key,
        data_nonce: data_nonce.to_vec(),
        ciphertext,
    })
    .map_err(|_| CryptoError::SerializationFailed)
}

pub fn decrypt_container(password: &[u8], container: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let parsed: EncryptedContainer =
        serde_json::from_slice(container).map_err(|_| CryptoError::AuthenticationFailed)?;
    if parsed.version != FORMAT_VERSION
        || parsed.salt.len() != SALT_SIZE
        || parsed.wrap_nonce.len() != NONCE_SIZE
        || parsed.data_nonce.len() != NONCE_SIZE
    {
        return Err(CryptoError::AuthenticationFailed);
    }

    let derived_key = derive_key(
        password,
        &parsed.salt,
        KdfParams {
            memory_kib: parsed.memory_kib,
            iterations: parsed.iterations,
            parallelism: parsed.parallelism,
        },
    )
    .map_err(|_| CryptoError::AuthenticationFailed)?;
    let wrapping_cipher = XChaCha20Poly1305::new(Key::from_slice(&derived_key));
    let data_key = wrapping_cipher
        .decrypt(
            XNonce::from_slice(&parsed.wrap_nonce),
            chacha20poly1305::aead::Payload {
                msg: &parsed.wrapped_key,
                aad: DATA_KEY_AAD,
            },
        )
        .map_err(|_| CryptoError::AuthenticationFailed)?;
    if data_key.len() != KEY_SIZE {
        return Err(CryptoError::AuthenticationFailed);
    }

    let data_cipher = XChaCha20Poly1305::new(Key::from_slice(&data_key));
    data_cipher
        .decrypt(
            XNonce::from_slice(&parsed.data_nonce),
            chacha20poly1305::aead::Payload {
                msg: &parsed.ciphertext,
                aad: PAYLOAD_AAD,
            },
        )
        .map_err(|_| CryptoError::AuthenticationFailed)
}

pub fn validate_container(container: &[u8]) -> Result<ContainerMetadata, CryptoError> {
    let parsed: EncryptedContainer =
        serde_json::from_slice(container).map_err(|_| CryptoError::InvalidContainer)?;
    if parsed.version != FORMAT_VERSION {
        return Err(CryptoError::UnsupportedVersion);
    }
    Ok(ContainerMetadata {
        version: parsed.version,
    })
}
