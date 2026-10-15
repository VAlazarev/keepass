export type SyncStatus =
  | "offline"
  | "synchronized"
  | "pending"
  | "remote_changes"
  | "conflict"
  | "authentication_required";

export interface Group {
  id: string;
  parentId: string | null;
  name: string;
  icon?: string;
  color?: string;
}

export interface CustomField {
  key: string;
  value: string;
  protected: boolean;
}

export interface VaultEntry {
  id: string;
  groupId: string;
  title: string;
  username: string;
  password: string;
  url: string;
  additionalUrls: string[];
  notes: string;
  tags: string[];
  customFields: CustomField[];
  favorite: boolean;
  createdAt: string;
  modifiedAt: string;
}

export interface VaultSummary {
  path: string;
  name: string;
  entryCount: number;
  groupCount: number;
  syncStatus: SyncStatus;
}

export interface VaultData {
  schemaVersion: number;
  groups: Group[];
  entries: VaultEntry[];
}
