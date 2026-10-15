import { FormEvent, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { APP_NAME } from "./app-config";
import { filterEntries } from "./lib/catalog";
import { copyWithTimeout } from "./lib/clipboard";
import { describeCommandError } from "./lib/command-error";
import type { Group, VaultData, VaultEntry, VaultSummary } from "./types/vault";
import "./styles.css";

const emptyData: VaultData = { schemaVersion: 1, groups: [], entries: [] };

function generatePassword() {
  const alphabet = "ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz23456789!@#$%";
  const values = new Uint32Array(20);
  crypto.getRandomValues(values);
  return Array.from(values, (value) => alphabet[value % alphabet.length]).join("");
}

export default function App() {
  const [summary, setSummary] = useState<VaultSummary | null>(null);
  const [data, setData] = useState<VaultData>(emptyData);
  const [path, setPath] = useState("C:\\Users\\Public\\keepass-vault.kdv");
  const [masterPassword, setMasterPassword] = useState("");
  const [error, setError] = useState("");
  const [query, setQuery] = useState("");
  const [selectedGroup, setSelectedGroup] = useState<string | undefined>();
  const [selectedId, setSelectedId] = useState<string | undefined>();
  const [showFavorites, setShowFavorites] = useState(false);
  const [editorOpen, setEditorOpen] = useState(false);
  const [newEntry, setNewEntry] = useState({ title: "", username: "", password: "", url: "", notes: "" });

  const filteredEntries = useMemo(
    () => filterEntries(data.entries, query, selectedGroup, showFavorites),
    [data.entries, query, selectedGroup, showFavorites],
  );
  const selectedEntry = data.entries.find((entry) => entry.id === selectedId) ?? filteredEntries[0];

  async function loadData() {
    const vault = await invoke<VaultData>("get_vault_data");
    setData(vault);
    setSelectedId(vault.entries[0]?.id);
  }

  async function openVault(create: boolean) {
    setError("");
    if (!masterPassword) {
      setError("Введите мастер-пароль");
      return;
    }
    try {
      const command = create ? "create_vault" : "unlock_vault";
      const nextSummary = await invoke<VaultSummary>(command, { path, password: masterPassword });
      setSummary(nextSummary);
      await loadData();
      setMasterPassword("");
    } catch (cause) {
      const reason = describeCommandError(cause);
      setError(create ? `Не удалось создать хранилище: ${reason}` : `Не удалось открыть хранилище: ${reason}`);
    }
  }

  async function lock() {
    await invoke("lock_vault");
    setSummary(null);
    setData(emptyData);
    setSelectedId(undefined);
  }

  async function submitEntry(event: FormEvent) {
    event.preventDefault();
    const groupId = selectedGroup ?? data.groups[0]?.id;
    if (!groupId || !newEntry.title.trim()) return;
    try {
      const entry = await invoke<VaultEntry>("create_entry", {
        input: {
          groupId,
          title: newEntry.title,
          username: newEntry.username,
          password: newEntry.password,
          url: newEntry.url,
          additionalUrls: [],
          notes: newEntry.notes,
          tags: [],
          customFields: [],
        },
      });
      setData((current) => ({ ...current, entries: [...current.entries, entry] }));
      setSelectedId(entry.id);
      setEditorOpen(false);
      setNewEntry({ title: "", username: "", password: "", url: "", notes: "" });
    } catch (cause) {
      setError(`Не удалось сохранить запись: ${describeCommandError(cause)}`);
    }
  }

  if (!summary) {
    return (
      <main className="unlock-page">
        <section className="unlock-card">
          <div className="brand-mark">🔐</div>
          <p className="eyebrow">Локальное защищённое хранилище</p>
          <h1>{APP_NAME}</h1>
          <p className="muted">Пароли шифруются до синхронизации с Google Drive.</p>
          <label>Файл хранилища<input value={path} onChange={(event) => setPath(event.target.value)} /></label>
          <label>Мастер-пароль<input type="password" value={masterPassword} onChange={(event) => setMasterPassword(event.target.value)} onKeyDown={(event) => event.key === "Enter" && openVault(false)} /></label>
          {error && <div className="error-banner">{error}</div>}
          <div className="unlock-actions"><button className="primary" onClick={() => openVault(false)}>Открыть хранилище</button><button onClick={() => openVault(true)}>Создать новое</button></div>
          <small>Мастер-пароль не отправляется в Google Drive и не восстанавливается.</small>
        </section>
      </main>
    );
  }

  return (
    <main className="app-shell">
      <header className="topbar"><div className="topbar-title"><span className="mini-mark">🔐</span><strong>{summary.name}</strong><span className="sync-pill">● Локально</span></div><div className="toolbar"><button onClick={() => setEditorOpen(true)}>＋ Новая запись</button><button onClick={() => selectedEntry && copyWithTimeout(selectedEntry.password)}>⧉ Копировать пароль</button><button onClick={lock}>🔒 Заблокировать</button></div></header>
      <div className="workspace">
        <aside className="sidebar"><div className="sidebar-heading"><span>КАТАЛОГ</span><button title="Добавить группу">＋</button></div><button className={!selectedGroup && !showFavorites ? "tree-item active" : "tree-item"} onClick={() => { setSelectedGroup(undefined); setShowFavorites(false); }}>🗂 Все записи <span>{data.entries.length}</span></button><button className={showFavorites ? "tree-item active" : "tree-item"} onClick={() => { setShowFavorites(true); setSelectedGroup(undefined); }}>★ Избранное</button><div className="tree-label">ГРУППЫ</div>{data.groups.map((group: Group) => <button className={selectedGroup === group.id ? "tree-item active" : "tree-item"} key={group.id} onClick={() => { setSelectedGroup(group.id); setShowFavorites(false); }}>{group.icon ?? "📁"} {group.name}<span>{data.entries.filter((entry) => entry.groupId === group.id).length}</span></button>)}<div className="sidebar-bottom"><button className="tree-item">♻ Корзина</button><button className="tree-item">⚙ Настройки</button></div></aside>
        <section className="content"><div className="content-header"><div><p className="eyebrow">{selectedGroup ? data.groups.find((group) => group.id === selectedGroup)?.name : showFavorites ? "Избранное" : "Все записи"}</p><h2>{filteredEntries.length} записей</h2></div><input className="search" placeholder="Поиск по названию, логину, URL, заметкам..." value={query} onChange={(event) => setQuery(event.target.value)} /></div><div className="entry-table"><div className="table-head"><span>Название</span><span>Логин</span><span>URL</span><span>Заметки</span></div>{filteredEntries.map((entry) => <button className={selectedEntry?.id === entry.id ? "entry-row selected" : "entry-row"} key={entry.id} onClick={() => setSelectedId(entry.id)}><span className="entry-title"><span className="entry-icon">{entry.favorite ? "★" : "●"}</span><strong>{entry.title}</strong></span><span>{entry.username || "—"}</span><span className="url-cell">{entry.url || "—"}</span><span>{entry.notes || "—"}</span></button>)}{filteredEntries.length === 0 && <div className="empty-state"><span>🗃</span><strong>Записей пока нет</strong><p>Добавьте первую запись в это представление.</p></div>}</div></section>
        <aside className="details">{selectedEntry ? <><div className="details-head"><div className="large-icon">{selectedEntry.favorite ? "★" : "🔑"}</div><div><h2>{selectedEntry.title}</h2><a href={selectedEntry.url} target="_blank">{selectedEntry.url || "Без URL"}</a></div><button>⋮</button></div><div className="field"><span>ЛОГИН</span><div><strong>{selectedEntry.username || "Не указан"}</strong><button onClick={() => copyWithTimeout(selectedEntry.username)}>⧉</button></div></div><div className="field"><span>ПАРОЛЬ</span><div><strong>••••••••••••</strong><button onClick={() => copyWithTimeout(selectedEntry.password)}>⧉</button></div></div><div className="field"><span>ЗАМЕТКИ</span><p>{selectedEntry.notes || "Нет заметок"}</p></div><div className="tag-list">{selectedEntry.tags.map((tag) => <span key={tag}>#{tag}</span>)}</div><button className="edit-button">✎ Редактировать запись</button></> : <div className="details-empty">Выберите запись, чтобы увидеть детали</div>}</aside>
      </div>
      {editorOpen && <div className="modal-backdrop"><form className="modal" onSubmit={submitEntry}><div className="modal-head"><h2>Новая запись</h2><button type="button" onClick={() => setEditorOpen(false)}>×</button></div><label>Название<input autoFocus value={newEntry.title} onChange={(event) => setNewEntry({ ...newEntry, title: event.target.value })} /></label><label>Логин<input value={newEntry.username} onChange={(event) => setNewEntry({ ...newEntry, username: event.target.value })} /></label><label>Пароль<div className="input-with-action"><input type="text" value={newEntry.password} onChange={(event) => setNewEntry({ ...newEntry, password: event.target.value })} /><button type="button" onClick={() => setNewEntry({ ...newEntry, password: generatePassword() })}>Сгенерировать</button></div></label><label>URL<input value={newEntry.url} onChange={(event) => setNewEntry({ ...newEntry, url: event.target.value })} /></label><label>Заметки<textarea value={newEntry.notes} onChange={(event) => setNewEntry({ ...newEntry, notes: event.target.value })} /></label>{error && <div className="error-banner">{error}</div>}<div className="modal-actions"><button type="button" onClick={() => setEditorOpen(false)}>Отмена</button><button className="primary" type="submit">Сохранить запись</button></div></form></div>}
    </main>
  );
}
