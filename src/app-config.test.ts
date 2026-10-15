import { describe, expect, it } from "vitest";
import { APP_NAME, DEFAULT_WINDOW } from "./app-config";

describe("application bootstrap", () => {
  it("exposes the product identity and default window size", () => {
    expect(APP_NAME).toBe("Keepass Drive Vault");
    expect(DEFAULT_WINDOW).toEqual({ width: 1280, height: 800 });
  });
});
