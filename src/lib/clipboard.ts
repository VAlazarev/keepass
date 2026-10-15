export interface ClipboardWriter {
  writeText(value: string): Promise<void>;
}

export async function copyWithTimeout(
  value: string,
  clipboard: ClipboardWriter = navigator.clipboard,
  timeoutMs = 15_000,
): Promise<void> {
  await clipboard.writeText(value);
  globalThis.setTimeout(() => void clipboard.writeText(""), timeoutMs);
}
