import { invoke as tauriInvoke } from "@tauri-apps/api/core";

export class DesktopRuntimeUnavailableError extends Error {
  constructor() {
    super(
      "Bergag's desktop runtime is unavailable. Open the app with 'npm run tauri dev' or a installed build.",
    );
    this.name = "DesktopRuntimeUnavailableError";
  }
}

export function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export async function invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauriRuntime()) {
    throw new DesktopRuntimeUnavailableError();
  }
  return tauriInvoke<T>(command, args);
}
