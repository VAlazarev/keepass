use chrono::Utc;
use uuid::Uuid;

use super::model::{Entry, EntryInput, Group, GroupInput, PasswordRevision};

#[derive(Debug, thiserror::Error)]
pub enum VaultError {
    #[error("group not found")]
    GroupNotFound,
    #[error("entry not found")]
    EntryNotFound,
    #[error("invalid input")]
    InvalidInput,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Vault {
    pub schema_version: u32,
    pub groups: Vec<Group>,
    pub entries: Vec<Entry>,
}

fn now() -> String {
    Utc::now().to_rfc3339()
}

fn id() -> String {
    Uuid::new_v4().to_string()
}

impl Vault {
    pub fn new() -> Self {
        let names = [
            ("Internet", "🌐"),
            ("Coding", "💻"),
            ("Gaming", "🎮"),
            ("Shopping", "🛒"),
            ("Social", "👥"),
            ("My Computer", "🖥"),
            ("Real World", "🏠"),
        ];
        let groups = names
            .into_iter()
            .map(|(name, icon)| Group {
                id: id(),
                parent_id: None,
                name: name.into(),
                icon: Some(icon.into()),
                color: None,
            })
            .collect();
        Self {
            schema_version: 1,
            groups,
            entries: Vec::new(),
        }
    }

    pub fn create_group(&mut self, input: GroupInput) -> Result<String, VaultError> {
        if input.name.trim().is_empty() {
            return Err(VaultError::InvalidInput);
        }
        if let Some(parent_id) = &input.parent_id {
            if !self.groups.iter().any(|group| &group.id == parent_id) {
                return Err(VaultError::GroupNotFound);
            }
        }
        let group_id = id();
        self.groups.push(Group {
            id: group_id.clone(),
            parent_id: input.parent_id,
            name: input.name,
            icon: input.icon,
            color: input.color,
        });
        Ok(group_id)
    }

    pub fn create_entry(&mut self, input: EntryInput) -> Result<String, VaultError> {
        if !self.groups.iter().any(|group| group.id == input.group_id) {
            return Err(VaultError::GroupNotFound);
        }
        if input.title.trim().is_empty() {
            return Err(VaultError::InvalidInput);
        }
        let timestamp = now();
        let entry_id = id();
        self.entries.push(Entry {
            id: entry_id.clone(),
            group_id: input.group_id,
            title: input.title,
            username: input.username,
            password: input.password,
            url: input.url,
            additional_urls: input.additional_urls,
            notes: input.notes,
            tags: input.tags,
            custom_fields: input.custom_fields,
            attachments: Vec::new(),
            password_history: Vec::new(),
            favorite: false,
            created_at: timestamp.clone(),
            modified_at: timestamp,
            last_used_at: None,
            expires_at: None,
            deleted_at: None,
        });
        Ok(entry_id)
    }

    pub fn update_entry(&mut self, entry_id: &str, input: EntryInput) -> Result<(), VaultError> {
        if !self.groups.iter().any(|group| group.id == input.group_id) {
            return Err(VaultError::GroupNotFound);
        }
        let entry = self
            .entries
            .iter_mut()
            .find(|entry| entry.id == entry_id)
            .ok_or(VaultError::EntryNotFound)?;
        if entry.password != input.password {
            entry.password_history.push(PasswordRevision {
                password: entry.password.clone(),
                changed_at: now(),
            });
        }
        entry.group_id = input.group_id;
        entry.title = input.title;
        entry.username = input.username;
        entry.password = input.password;
        entry.url = input.url;
        entry.additional_urls = input.additional_urls;
        entry.notes = input.notes;
        entry.tags = input.tags;
        entry.custom_fields = input.custom_fields;
        entry.modified_at = now();
        Ok(())
    }

    pub fn find_entry(&self, entry_id: &str) -> Result<&Entry, VaultError> {
        self.entries
            .iter()
            .find(|entry| entry.id == entry_id)
            .ok_or(VaultError::EntryNotFound)
    }

    pub fn delete_entry(&mut self, entry_id: &str) -> Result<(), VaultError> {
        let entry = self
            .entries
            .iter_mut()
            .find(|entry| entry.id == entry_id)
            .ok_or(VaultError::EntryNotFound)?;
        entry.deleted_at = Some(now());
        entry.modified_at = now();
        Ok(())
    }

    pub fn restore_entry(&mut self, entry_id: &str) -> Result<(), VaultError> {
        let entry = self
            .entries
            .iter_mut()
            .find(|entry| entry.id == entry_id)
            .ok_or(VaultError::EntryNotFound)?;
        entry.deleted_at = None;
        entry.modified_at = now();
        Ok(())
    }
}

impl Default for Vault {
    fn default() -> Self {
        Self::new()
    }
}
