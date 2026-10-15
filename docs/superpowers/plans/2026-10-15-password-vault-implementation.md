# Windows Password Vault Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Windows desktop password vault with local authenticated encryption, KeePass-like cataloging, and encrypted Google Drive synchronization.

**Architecture:** Tauri hosts a React/TypeScript UI and a Rust core. The Rust core owns key derivation, encryption, vault serialization, local persistence, and sync orchestration; the UI communicates through typed Tauri commands and never receives the master key. The first implementation delivers a usable local vault before enabling Google Drive and browser integration.

**Tech Stack:** Tauri 2, Rust, React, TypeScript, SQLite, Argon2id, XChaCha20-Poly1305, Google Drive API/OAuth 2.0, Vitest, Rust unit tests, Playwright or Tauri UI tests.

**Spec:** `docs/superpowers/specs/2026-10-15-windows-password-vault-design.md`

## Global Constraints

- The application is Windows-first and must work offline.
- Google Drive receives ciphertext only; the master password and derived keys never leave the local vault core.
- Use audited cryptographic libraries; do not implement custom cryptography.
- Do not log passwords, keys, decrypted payloads, or OAuth secrets.
- Failed authentication, tampering, or corrupted vault data must fail closed.
- Browser autofill is phase 4 and must not block the secure local vault or Drive sync.
- Destructive operations require confirmation and deleted entries use a recycle bin.

## Review Focus

- Wrong master password or tampered ciphertext must return a generic unlock failure and expose no partial data; test in Task 2.
- Interrupted local writes must preserve the last valid vault; test in Task 3.
- Remote/local revision conflicts must never silently overwrite data; test in Task 6.
- Clipboard contents must be cleared after the configured timeout and on lock; test in Task 5.
- Import data containing duplicates, malformed rows, or empty passwords must be previewed and rejected or resolved explicitly; test in Task 5.

### Task 1: Project bootstrap and typed boundaries

**Files:**
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/src/main.rs`
- Create: `src-tauri/src/commands/mod.rs`
- Create: `src-tauri/src/core/mod.rs`
- Create: `src-tauri/src/storage/mod.rs`
- Create: `src-tauri/src/sync/mod.rs`
- Create: `src/App.tsx`
- Create: `src/main.tsx`
- Create: `src/types/vault.ts`
- Create: `package.json`
- Create: `vite.config.ts`
- Create: `tsconfig.json`
- Create: `index.html`

**Interfaces:**
- Produces Rust module boundaries for `core`, `storage`, `sync`, and `commands`.
- Produces TypeScript types `VaultSummary`, `Group`, `VaultEntry`, `CustomField`, and `SyncStatus`.

- [ ] **Step 1: Verify local toolchain availability**

Run `node --version`, `npm --version`, `rustc --version`, and `cargo --version`. Record the available versions in the setup notes if they differ from the selected Tauri requirements.

- [ ] **Step 2: Scaffold the Tauri/Vite application**

Create the minimal app shell and ensure the default window opens on Windows.

- [ ] **Step 3: Define shared domain types**

Define stable serialized fields for groups, entries, tags, URLs, custom fields, attachments, revisions, and sync status. Keep secret values represented only inside Rust vault-core structures; TypeScript receives entry data only after unlock.

- [ ] **Step 4: Add a smoke test and run the app**

Verify `npm run build` and `cargo test` succeed before adding product behavior.

- [ ] **Step 5: Commit**

Commit as `chore: scaffold password vault application`.

### Task 2: Vault encryption core

**Files:**
- Create: `src-tauri/src/core/crypto.rs`
- Create: `src-tauri/src/core/container.rs`
- Create: `src-tauri/src/core/error.rs`
- Create: `src-tauri/tests/crypto_tests.rs`

**Interfaces:**
- `derive_key(password: &[u8], salt: &[u8], params: KdfParams) -> Result<[u8; 32], CryptoError>`
- `encrypt_container(password: &[u8], payload: &[u8]) -> Result<Vec<u8>, CryptoError>`
- `decrypt_container(password: &[u8], container: &[u8]) -> Result<Vec<u8>, CryptoError>`
- `validate_container(container: &[u8]) -> Result<ContainerMetadata, CryptoError>`

- [ ] **Step 1: Write failing Rust tests**

Test round-trip encryption, wrong-password rejection, ciphertext tamper rejection, non-deterministic ciphertext for identical plaintext, and unsupported container version rejection.

- [ ] **Step 2: Run the tests and verify they fail**

Run `cargo test -p keepass --test crypto_tests` or the project-equivalent command. Expected: compile/test failure because the crypto interfaces do not exist.

- [ ] **Step 3: Implement the versioned authenticated container**

Use Argon2id for password-based key derivation and XChaCha20-Poly1305 for authenticated encryption. Store the format version, KDF parameters, salt, nonce, wrapped metadata, ciphertext, and authentication data. Return a single generic authentication error for wrong password and tampering.

- [ ] **Step 4: Run the tests and verify they pass**

Run `cargo test --all`. Expected: all crypto tests pass.

- [ ] **Step 5: Commit**

Commit as `feat: add authenticated vault encryption`.

### Task 3: Local vault model and crash-safe storage

**Files:**
- Create: `src-tauri/src/core/model.rs`
- Create: `src-tauri/src/core/vault.rs`
- Create: `src-tauri/src/storage/local.rs`
- Create: `src-tauri/tests/vault_tests.rs`

**Interfaces:**
- `Vault::new() -> Vault`
- `Vault::create_group(&mut self, input: GroupInput) -> Result<GroupId, VaultError>`
- `Vault::create_entry(&mut self, input: EntryInput) -> Result<EntryId, VaultError>`
- `Vault::update_entry(&mut self, id: EntryId, input: EntryInput) -> Result<(), VaultError>`
- `Vault::move_entry(&mut self, id: EntryId, group_id: GroupId) -> Result<(), VaultError>`
- `Vault::delete_entry(&mut self, id: EntryId) -> Result<(), VaultError>`
- `Vault::restore_entry(&mut self, id: EntryId) -> Result<(), VaultError>`
- `LocalVaultStore::save(&self, vault: &Vault, password: &[u8]) -> Result<(), StorageError>`
- `LocalVaultStore::load(&self, password: &[u8]) -> Result<Vault, StorageError>`

- [ ] **Step 1: Write failing model and storage tests**

Cover default groups, entry CRUD, custom fields, tags, recycle-bin restore, revision timestamps, serialization round trips, and preservation of the last valid file after a simulated interrupted write.

- [ ] **Step 2: Run the tests and verify they fail**

Run `cargo test --all`. Expected: failure because model and storage interfaces are not implemented.

- [ ] **Step 3: Implement the domain model**

Use stable IDs, explicit group relationships, entry revisions, and soft deletion. Keep password history and attachments represented in the encrypted vault payload.

- [ ] **Step 4: Implement crash-safe local persistence**

Serialize the vault, encrypt it with Task 2, write to a temporary file in the same directory, flush it, atomically replace the destination, and retain a local recovery copy before replacement.

- [ ] **Step 5: Run tests and commit**

Run `cargo test --all`; expected: PASS. Commit as `feat: add local vault model and persistence`.

### Task 4: Unlock flow and basic Tauri commands

**Files:**
- Create: `src-tauri/src/commands/vault_commands.rs`
- Modify: `src-tauri/src/main.rs`
- Create: `src-tauri/src/session.rs`
- Create: `src-tauri/tests/command_tests.rs`
- Modify: `src/types/vault.ts`

**Interfaces:**
- `create_vault(path: String, password: String) -> CommandResult<VaultSummary>`
- `unlock_vault(path: String, password: String) -> CommandResult<VaultSummary>`
- `lock_vault() -> CommandResult<()>`
- `get_vault_snapshot() -> CommandResult<VaultSnapshot>`
- `save_vault() -> CommandResult<SaveResult>`
- `upsert_entry(input: EntryInput) -> CommandResult<VaultEntry>`
- `delete_entry(id: String) -> CommandResult<()>`

- [ ] **Step 1: Write failing command/session tests**

Test create, unlock, wrong-password failure, lock clearing the session, and refusal to call data commands while locked.

- [ ] **Step 2: Implement an in-memory locked/unlocked session**

Keep the decrypted vault and key inside Rust state only. Tauri commands return sanitized snapshots and never return the master key or raw container.

- [ ] **Step 3: Register commands and test through the command boundary**

Return structured errors suitable for UI messages without revealing whether a password or file path was the sensitive failure.

- [ ] **Step 4: Run tests and commit**

Run `cargo test --all` and `npm run build`. Commit as `feat: add vault unlock and command API`.

### Task 5: KeePass-like catalog UI and local features

**Files:**
- Create: `src/components/UnlockScreen.tsx`
- Create: `src/components/VaultLayout.tsx`
- Create: `src/components/GroupTree.tsx`
- Create: `src/components/EntryTable.tsx`
- Create: `src/components/EntryEditor.tsx`
- Create: `src/components/PasswordGenerator.tsx`
- Create: `src/components/ImportExportDialog.tsx`
- Create: `src/lib/clipboard.ts`
- Create: `src/lib/password-health.ts`
- Modify: `src/App.tsx`
- Create: `src/**/*.test.tsx`

**Interfaces:**
- UI consumes typed Tauri commands from Task 4.
- UI produces the left group tree, central entry table, details/editor pane, search, filters, generator, and lock controls.

- [ ] **Step 1: Write failing UI tests**

Test unlock states, tree selection, search across title/username/URL/notes/tags, create/edit/delete/restore, hidden/revealed password, copy-and-clear clipboard, generator constraints, and import preview validation.

- [ ] **Step 2: Implement the application shell**

Match the supplied reference layout: left catalog tree, central entries table, and right-side details/editor area. Add responsive behavior for narrow windows and dark/light themes.

- [ ] **Step 3: Implement entry editor and catalog actions**

Support standard fields, custom key/value fields, multiple URLs, tags, groups, notes, password history, attachments metadata, favorites, archive, and recycle bin.

- [ ] **Step 4: Implement security-sensitive UI behavior**

Use timed clipboard clearing, lock on inactivity and Windows workstation lock notification, confirmation dialogs for destructive actions, and no plaintext values in UI error telemetry.

- [ ] **Step 5: Run UI tests and commit**

Run `npm test`, `npm run build`, and the Tauri dev build. Commit as `feat: add password vault catalog interface`.

### Task 6: Google Drive encrypted synchronization

**Files:**
- Create: `src-tauri/src/sync/google_drive.rs`
- Create: `src-tauri/src/sync/conflicts.rs`
- Create: `src-tauri/src/commands/sync_commands.rs`
- Create: `src-tauri/tests/sync_tests.rs`
- Create: `src/components/SyncCenter.tsx`
- Modify: `src/types/vault.ts`

**Interfaces:**
- `GoogleDriveSync::authorize() -> Result<AccountInfo, SyncError>`
- `GoogleDriveSync::pull() -> Result<RemoteVault, SyncError>`
- `GoogleDriveSync::push(container: &[u8], base_revision: Option<&str>) -> Result<SyncReceipt, SyncError>`
- `SyncCoordinator::status() -> SyncStatus`
- `SyncCoordinator::sync_now() -> Result<SyncStatus, SyncError>`
- `SyncCoordinator::resolve_conflict(choice: ConflictChoice) -> Result<SyncStatus, SyncError>`

- [ ] **Step 1: Write failing sync tests with a fake Drive provider**

Cover first upload, download on a second installation, offline pending state, OAuth expiration, remote revision conflict, keep-local/keep-remote/save-both resolution, and ciphertext-only upload assertions.

- [ ] **Step 2: Implement a Drive provider boundary**

Keep Google-specific HTTP/OAuth code behind a trait so sync logic can be tested without network access. Limit Drive access to the app-owned folder scope selected in the design.

- [ ] **Step 3: Implement revision-aware sync**

Use remote revision IDs and content hashes, create recoverable previous versions before replacement, and refuse silent overwrites. Persist only encrypted containers remotely.

- [ ] **Step 4: Add sync commands and UI**

Show synchronized, pending, remote changes, conflict, and authentication failure states with explicit recovery actions.

- [ ] **Step 5: Run tests and commit**

Run `cargo test --all`, `npm test`, and a manual OAuth/Drive smoke test using a test Google project. Commit as `feat: add encrypted Google Drive synchronization`.

### Task 7: Hardening, packaging, and release checks

**Files:**
- Create: `src-tauri/src/integration/windows.rs`
- Create: `tests/e2e/vault.spec.ts`
- Create: `README.md`
- Create: `.github/workflows/build.yml` or equivalent local build script
- Modify: `src-tauri/tauri.conf.json`

**Interfaces:**
- Produces a Windows installer, documented recovery workflow, and repeatable verification commands.

- [ ] **Step 1: Add end-to-end tests**

Test first launch, create/unlock/lock, entry lifecycle, backup restore, sync status, and clean shutdown.

- [ ] **Step 2: Add Windows integration**

Handle workstation lock notifications, global shortcuts, application data paths, startup behavior, and secure default file permissions.

- [ ] **Step 3: Audit sensitive output**

Search logs, temporary directories, build artifacts, and sync fixtures for plaintext password values or OAuth secrets. Verify locked state clears in-memory session data as far as the platform permits.

- [ ] **Step 4: Build and package**

Run the complete Rust and TypeScript test suites, build a Windows installer, and test installation on a clean Windows environment.

- [ ] **Step 5: Commit**

Commit as `chore: harden and package password vault`.

## Execution order

Tasks must be completed in order: 1 → 2 → 3 → 4 → 5 → 6 → 7. Browser autofill is intentionally deferred until after these tasks and should receive a separate design and plan because it introduces an additional trust boundary and browser-specific testing.
