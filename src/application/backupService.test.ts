import { describe, expect, it } from "vitest";
import {
  backupCreatedMessage,
  backupDeleteMessage,
  backupLabelOrDefault,
  backupRestoreMessage,
  backupRestoredMessage,
  backupVerificationMessage,
  formatBackupSize,
} from "./backupService";
import type { BackupRecord } from "../domain/models";

const backup: BackupRecord = {
  id: "1e6f4a7c-6b62-4b3e-9c1a-4b8f0f2f5d91",
  label: "Manual backup",
  created_at: "2026-11-20T12:00:00Z",
  schema_version: 8,
  profile_count: 2,
  size_bytes: 2_359_296,
  automatic: false,
};

describe("backupService", () => {
  it("formats real backup sizes", () => {
    expect(formatBackupSize(512)).toBe("512 B");
    expect(formatBackupSize(2048)).toBe("2.0 KB");
    expect(formatBackupSize(2_359_296)).toBe("2.3 MB");
  });

  it("explains that deletion removes only app-created backup files", () => {
    expect(backupDeleteMessage(backup)).toContain("removes only the backup files Bergag created");
  });

  it("warns that restore replaces the library and preserves credentials", () => {
    const message = backupRestoreMessage(backup);
    expect(message).toContain("current library will be replaced");
    expect(message).toContain("safety backup");
    expect(message).toContain("credentials in Windows Credential Manager are not part");
  });

  it("reports real creation, verification, and restore outcomes", () => {
    expect(backupCreatedMessage(backup)).toContain("stored locally");
    expect(backupVerificationMessage({ valid: true, reason: null })).toContain(
      "integrity check all passed",
    );
    expect(backupVerificationMessage({ valid: false, reason: "checksum mismatch" })).toContain(
      "current library was not changed",
    );
    expect(
      backupRestoredMessage({ safety_backup_id: "safety", schema_version: 8, profile_count: 1 }),
    ).toContain("1 profile available at schema 8");
  });

  it("creates a non-empty default label", () => {
    expect(backupLabelOrDefault("  ")).toMatch(/^Manual backup /);
    expect(backupLabelOrDefault("  Nightly  ")).toBe("Nightly");
  });
});
