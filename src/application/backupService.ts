import type { BackupRecord, BackupVerification, RestoreOutcome } from "../domain/models";

export function formatBackupSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export function formatBackupDate(timestamp: string): string {
  const parsed = new Date(timestamp);
  if (Number.isNaN(parsed.getTime())) return timestamp;
  return parsed.toLocaleString();
}

export function backupDeleteMessage(backup: BackupRecord): string {
  return `Delete the backup "${backup.label}" from this device? This removes only the backup files Bergag created and cannot be undone.`;
}

export function backupRestoreMessage(backup: BackupRecord): string {
  return `Restore "${backup.label}" now? Your current library will be replaced with the backup contents. Bergag will first create a safety backup of the current library, then restart is not required. Saved credentials in Windows Credential Manager are not part of the backup.`;
}

export function backupCreatedMessage(backup: BackupRecord): string {
  return `Verified backup "${backup.label}" stored locally.`;
}

export function backupVerificationMessage(result: BackupVerification): string {
  if (result.valid)
    return "Backup checksum, manifest, profile list, and integrity check all passed.";
  return `Backup verification failed: ${result.reason ?? "unknown reason"}. The current library was not changed.`;
}

export function backupRestoredMessage(outcome: RestoreOutcome): string {
  return `Restore finished. ${outcome.profile_count} profile${outcome.profile_count === 1 ? "" : "s"} available at schema ${outcome.schema_version}. A safety backup was created first.`;
}

export function backupLabelOrDefault(raw: string): string {
  return raw.trim() || `Manual backup ${formatBackupDate(new Date().toISOString())}`;
}
