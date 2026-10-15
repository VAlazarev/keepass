use keepass_drive_vault_lib::core::model::{EntryInput, GroupInput};
use keepass_drive_vault_lib::core::vault::Vault;
use keepass_drive_vault_lib::storage::local::LocalVaultStore;

#[test]
fn new_vault_contains_the_default_catalog() {
    let vault = Vault::new();
    let names: Vec<_> = vault
        .groups
        .iter()
        .map(|group| group.name.as_str())
        .collect();
    assert!(names.contains(&"Internet"));
    assert!(names.contains(&"Coding"));
    assert!(names.contains(&"Gaming"));
    assert!(names.contains(&"Shopping"));
    assert!(names.contains(&"Social"));
    assert!(names.contains(&"My Computer"));
    assert!(names.contains(&"Real World"));
}

#[test]
fn entry_can_be_created_updated_deleted_and_restored() {
    let mut vault = Vault::new();
    let group_id = vault
        .create_group(GroupInput {
            name: "Personal".into(),
            parent_id: None,
            icon: None,
            color: None,
        })
        .unwrap();
    let entry_id = vault
        .create_entry(EntryInput {
            group_id: group_id.clone(),
            title: "Example".into(),
            username: "alice".into(),
            password: "secret".into(),
            url: "https://example.com".into(),
            additional_urls: vec![],
            notes: "private".into(),
            tags: vec!["personal".into()],
            custom_fields: vec![],
        })
        .unwrap();

    vault
        .update_entry(
            &entry_id,
            EntryInput {
                group_id,
                title: "Updated".into(),
                username: "alice".into(),
                password: "new-secret".into(),
                url: "https://example.com".into(),
                additional_urls: vec![],
                notes: "changed".into(),
                tags: vec!["personal".into(), "important".into()],
                custom_fields: vec![],
            },
        )
        .unwrap();
    assert_eq!(vault.find_entry(&entry_id).unwrap().title, "Updated");
    assert_eq!(
        vault.find_entry(&entry_id).unwrap().password_history.len(),
        1
    );

    vault.delete_entry(&entry_id).unwrap();
    assert!(vault.find_entry(&entry_id).unwrap().deleted_at.is_some());
    vault.restore_entry(&entry_id).unwrap();
    assert!(vault.find_entry(&entry_id).unwrap().deleted_at.is_none());
}

#[test]
fn vault_round_trips_through_encrypted_local_storage() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("vault.kdv");
    let store = LocalVaultStore::new(&path);
    let mut original = Vault::new();
    let group_id = original.groups[0].id.clone();
    original
        .create_entry(EntryInput {
            group_id,
            title: "Round trip".into(),
            username: "user".into(),
            password: "password".into(),
            url: "https://example.test".into(),
            additional_urls: vec![],
            notes: String::new(),
            tags: vec![],
            custom_fields: vec![],
        })
        .unwrap();

    store.save(&original, b"master password").unwrap();
    let loaded = store.load(b"master password").unwrap();

    assert_eq!(loaded.entries.len(), 1);
    assert_eq!(loaded.entries[0].title, "Round trip");
}

#[test]
fn second_save_keeps_a_recovery_copy() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("vault.kdv");
    let store = LocalVaultStore::new(&path);
    let vault = Vault::new();

    store.save(&vault, b"master password").unwrap();
    store.save(&vault, b"master password").unwrap();

    assert!(path.with_extension("kdv.bak").exists());
}
