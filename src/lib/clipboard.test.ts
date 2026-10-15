import { afterEach, describe, expect, it, vi } from "vitest";
import { copyWithTimeout, type ClipboardWriter } from "./clipboard";

describe("clipboard protection", () => {
  afterEach(() => vi.useRealTimers());

  it("clears copied text after the configured timeout", async () => {
    vi.useFakeTimers();
    const writes: string[] = [];
    const clipboard: ClipboardWriter = { writeText: async (value) => void writes.push(value) };

    await copyWithTimeout("private password", clipboard, 5000);
    expect(writes).toEqual(["private password"]);

    vi.advanceTimersByTime(5000);
    await vi.runAllTimersAsync();
    expect(writes).toEqual(["private password", ""]);
  });
});
