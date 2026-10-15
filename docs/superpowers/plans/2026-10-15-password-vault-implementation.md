# План реализации менеджера паролей для Windows

> **Для исполнителя:** используйте `superpowers:executing-plans` для последовательной реализации задач. Все шаги отмечаются чекбоксами.

**Цель:** создать Windows-приложение с локальным зашифрованным хранилищем, каталогизацией уровня KeePassXC и синхронизацией зашифрованного файла через Google Drive.

**Архитектура:** Tauri объединяет React/TypeScript-интерфейс и Rust-ядро. Rust владеет ключами, шифрованием, локальным хранением и синхронизацией; интерфейс получает только данные после разблокировки. Сначала создаётся полноценное локальное хранилище, затем подключается Google Drive.

**Стек:** Tauri 2, Rust, React, TypeScript, SQLite, Argon2id, XChaCha20-Poly1305, Google Drive API/OAuth 2.0, Vitest и Rust tests.

**Спецификация:** `docs/superpowers/specs/2026-10-15-windows-password-vault-design.md`

## Общие ограничения

- Приложение ориентировано на Windows и работает без интернета.
- Google Drive получает только шифротекст.
- Мастер-пароль и ключи не покидают локальное ядро.
- Нельзя писать собственную криптографию.
- Нельзя писать пароли, ключи и OAuth-секреты в логи.
- Ошибки пароля, повреждение и подмена должны завершаться безопасным отказом.
- Автозаполнение браузера является отдельным этапом после локального хранилища и синхронизации.
- Удаление выполняется через корзину и требует подтверждения.

## Задача 1: Каркас проекта и границы модулей

**Файлы:** `package.json`, `vite.config.ts`, `tsconfig*.json`, `src/App.tsx`, `src/main.tsx`, `src/types/vault.ts`, `src-tauri/Cargo.toml`, `src-tauri/src/{main,lib}.rs`, `src-tauri/src/{commands,core,storage,sync}/mod.rs`, `src-tauri/tauri.conf.json`.

- [ ] Проверить Node/npm и Rust/Cargo.
- [ ] Создать минимальный Tauri/Vite-проект.
- [ ] Описать типы `Group`, `VaultEntry`, `CustomField`, `VaultSummary`, `SyncStatus`.
- [ ] Добавить smoke-проверки `npm run build` и `cargo test`.
- [ ] Зафиксировать коммитом `chore: scaffold password vault application`.

## Задача 2: Криптографическое ядро

**Файлы:** `src-tauri/src/core/crypto.rs`, `container.rs`, `error.rs`, `src-tauri/tests/crypto_tests.rs`.

Интерфейсы:

- `derive_key(password, salt, params) -> Result<[u8; 32], CryptoError>`;
- `encrypt_container(password, payload) -> Result<Vec<u8>, CryptoError>`;
- `decrypt_container(password, container) -> Result<Vec<u8>, CryptoError>`;
- `validate_container(container) -> Result<ContainerMetadata, CryptoError>`.

- [ ] Написать тесты round-trip, неверного пароля, подмены, случайного nonce и неизвестной версии.
- [ ] Убедиться, что тесты сначала падают из-за отсутствия реализации.
- [ ] Реализовать Argon2id и XChaCha20-Poly1305 в версионируемом контейнере.
- [ ] Убедиться, что неверный пароль и подмена дают общий отказ аутентификации.
- [ ] Запустить `cargo test --all` и создать коммит `feat: add authenticated vault encryption`.

## Задача 3: Модель хранилища и безопасная запись

**Файлы:** `src-tauri/src/core/model.rs`, `vault.rs`, `src-tauri/src/storage/local.rs`, `src-tauri/tests/vault_tests.rs`.

Интерфейсы:

- `Vault::new()`;
- `create_group`, `create_entry`, `update_entry`, `move_entry`;
- `delete_entry`, `restore_entry`;
- `LocalVaultStore::save` и `LocalVaultStore::load`.

- [ ] Написать тесты групп по умолчанию, CRUD, полей, тегов, корзины и истории.
- [ ] Написать тест прерванной записи и сохранения последней рабочей версии.
- [ ] Реализовать стабильные ID, мягкое удаление и сериализацию.
- [ ] Реализовать временный файл, flush и атомарную замену.
- [ ] Запустить `cargo test --all` и создать коммит `feat: add local vault model and persistence`.

## Задача 4: Разблокировка и Tauri-команды

**Файлы:** `src-tauri/src/commands/vault_commands.rs`, `src-tauri/src/session.rs`, `src-tauri/tests/command_tests.rs`, `src/types/vault.ts`.

Команды:

- `create_vault(path, password)`;
- `unlock_vault(path, password)`;
- `lock_vault()`;
- `get_vault_snapshot()`;
- `save_vault()`;
- `upsert_entry(input)`;
- `delete_entry(id)`.

- [ ] Написать тесты создания, разблокировки, отказа, блокировки и запрета команд в locked-состоянии.
- [ ] Реализовать сессию в памяти Rust без передачи ключа в UI.
- [ ] Зарегистрировать команды и структурированные безопасные ошибки.
- [ ] Запустить `cargo test --all`, `npm run build` и создать коммит `feat: add vault unlock and command API`.

## Задача 5: Интерфейс каталога и локальные функции

**Файлы:** `src/components/UnlockScreen.tsx`, `VaultLayout.tsx`, `GroupTree.tsx`, `EntryTable.tsx`, `EntryEditor.tsx`, `PasswordGenerator.tsx`, `ImportExportDialog.tsx`, `src/lib/clipboard.ts`, `src/lib/password-health.ts`, тесты интерфейса.

- [ ] Написать тесты разблокировки, дерева, поиска, CRUD, восстановления, генератора и preview импорта.
- [ ] Реализовать layout по референсу: дерево слева, таблица по центру, детали справа.
- [ ] Добавить стандартные и дополнительные поля, URL, теги, избранное, архив, корзину и вложения.
- [ ] Добавить таймер очистки буфера, блокировку по бездействию и подтверждение опасных действий.
- [ ] Запустить `npm test`, `npm run build` и создать коммит `feat: add password vault catalog interface`.

## Задача 6: Синхронизация с Google Drive

**Файлы:** `src-tauri/src/sync/google_drive.rs`, `conflicts.rs`, `src-tauri/src/commands/sync_commands.rs`, `src-tauri/tests/sync_tests.rs`, `src/components/SyncCenter.tsx`.

Интерфейсы:

- `GoogleDriveSync::authorize`, `pull`, `push`;
- `SyncCoordinator::status`, `sync_now`, `resolve_conflict`.

- [ ] Написать тесты fake Drive для upload, download, offline, OAuth expiration, конфликта и ciphertext-only payload.
- [ ] Скрыть Google API за тестируемым provider trait.
- [ ] Реализовать ревизии, хэши, резервные версии и запрет тихого overwrite.
- [ ] Добавить команды и UI состояний синхронизации.
- [ ] Запустить Rust/TypeScript-тесты и ручной OAuth smoke test.
- [ ] Создать коммит `feat: add encrypted Google Drive synchronization`.

## Задача 7: Укрепление и упаковка

**Файлы:** `src-tauri/src/integration/windows.rs`, `tests/e2e/vault.spec.ts`, `README.md`, `src-tauri/tauri.conf.json`, build script.

- [ ] Добавить end-to-end тесты запуска, CRUD, lock/unlock, backup и shutdown.
- [ ] Добавить обработку блокировки Windows, горячих клавиш и прав файлов.
- [ ] Проверить логи, временные файлы, fixture и payload на отсутствие открытых паролей.
- [ ] Собрать установщик и проверить его на чистой Windows-системе.
- [ ] Создать коммит `chore: harden and package password vault`.

Задачи выполняются по порядку: 1 → 2 → 3 → 4 → 5 → 6 → 7. Браузерное расширение требует отдельного дизайна и плана.
