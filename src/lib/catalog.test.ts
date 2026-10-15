import { describe, expect, it } from "vitest";
import { filterEntries } from "./catalog";
import type { VaultEntry } from "../types/vault";

const entries: VaultEntry[] = [
  {
    id: "1",
    groupId: "internet",
    title: "Google",
    username: "alice@example.com",
    password: "secret",
    url: "https://google.com",
    additionalUrls: [],
    notes: "personal mail",
    tags: ["mail"],
    customFields: [],
    favorite: false,
    createdAt: "2026-01-01T00:00:00Z",
    modifiedAt: "2026-01-01T00:00:00Z",
  },
  {
    id: "2",
    groupId: "coding",
    title: "GitHub",
    username: "alice",
    password: "secret",
    url: "https://github.com",
    additionalUrls: [],
    notes: "work account",
    tags: ["code"],
    customFields: [],
    favorite: true,
    createdAt: "2026-01-01T00:00:00Z",
    modifiedAt: "2026-01-01T00:00:00Z",
  },
];

describe("catalog filtering", () => {
  it("searches title, username, URL, notes, and tags", () => {
    expect(filterEntries(entries, "mail").map((entry) => entry.id)).toEqual(["1"]);
    expect(filterEntries(entries, "github").map((entry) => entry.id)).toEqual(["2"]);
    expect(filterEntries(entries, "alice").map((entry) => entry.id)).toEqual(["1", "2"]);
  });

  it("filters by group and favorites", () => {
    expect(filterEntries(entries, "", "coding").map((entry) => entry.id)).toEqual(["2"]);
    expect(filterEntries(entries, "", undefined, true).map((entry) => entry.id)).toEqual(["2"]);
  });
});
