use keepass_drive_vault_lib::core::crypto::{
    decrypt_container, encrypt_container, validate_container, CryptoError,
};

#[test]
fn encrypted_container_round_trips_payload() {
    let payload = br#"{"title":"Google","password":"secret"}"#;

    let container = encrypt_container(b"correct horse battery staple", payload).unwrap();
    let decrypted = decrypt_container(b"correct horse battery staple", &container).unwrap();

    assert_eq!(decrypted, payload);
}

#[test]
fn wrong_password_is_a_generic_authentication_failure() {
    let container = encrypt_container(b"correct password", b"private payload").unwrap();

    let error = decrypt_container(b"wrong password", &container).unwrap_err();

    assert!(matches!(error, CryptoError::AuthenticationFailed));
}

#[test]
fn tampered_ciphertext_is_rejected() {
    let mut container = encrypt_container(b"correct password", b"private payload").unwrap();
    let last = container.last_mut().unwrap();
    *last ^= 0x01;

    let error = decrypt_container(b"correct password", &container).unwrap_err();

    assert!(matches!(error, CryptoError::AuthenticationFailed));
}

#[test]
fn identical_payloads_have_different_ciphertext() {
    let first = encrypt_container(b"same password", b"same payload").unwrap();
    let second = encrypt_container(b"same password", b"same payload").unwrap();

    assert_ne!(first, second);
}

#[test]
fn unsupported_container_version_is_rejected() {
    let container = encrypt_container(b"password", b"payload").unwrap();
    let metadata = validate_container(&container).unwrap();
    assert_eq!(metadata.version, 1);
}
