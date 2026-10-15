use keepass_drive_vault_lib::session::VaultSession;

#[test]
fn session_can_create_unlock_and_lock_a_vault() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("session.kdv");
    let mut session = VaultSession::new();

    session.create(&path, b"master password").unwrap();
    assert!(session.is_unlocked());
    assert_eq!(session.snapshot().unwrap().entry_count, 0);

    session.lock();
    assert!(!session.is_unlocked());
    assert!(session.snapshot().is_err());

    session.unlock(&path, b"master password").unwrap();
    assert!(session.is_unlocked());
}

#[test]
fn wrong_password_does_not_unlock_the_session() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("session.kdv");
    let mut session = VaultSession::new();
    session.create(&path, b"master password").unwrap();
    session.lock();

    assert!(session.unlock(&path, b"wrong password").is_err());
    assert!(!session.is_unlocked());
}

#[test]
fn locked_session_rejects_data_access() {
    let session = VaultSession::new();

    assert!(session.snapshot().is_err());
    assert!(session.current_path().is_none());
}
