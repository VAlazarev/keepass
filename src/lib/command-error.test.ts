import { describe, expect, it } from "vitest";
import { describeCommandError } from "./command-error";

describe("command error messages", () => {
  it("uses the safe message returned by the Rust command", () => {
    expect(describeCommandError({ code: "VAULT_ERROR", message: "Хранилище заблокировано" }))
      .toBe("Хранилище заблокировано");
  });

  it("uses a textual Tauri transport error", () => {
    expect(describeCommandError("invalid args: missing field groupId"))
      .toBe("invalid args: missing field groupId");
  });

  it("falls back for an unknown error value", () => {
    expect(describeCommandError(null)).toBe("Неизвестная ошибка приложения");
  });
});
