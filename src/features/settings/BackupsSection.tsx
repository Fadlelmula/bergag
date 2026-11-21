import { errorMessage } from "../../application/articleService";
import {
  backupCreatedMessage,
  backupDeleteMessage,
  backupLabelOrDefault,
  backupRestoreMessage,
  backupRestoredMessage,
  backupVerificationMessage,
  formatBackupDate,
  formatBackupSize,
} from "../../application/backupService";
import { queryKeys } from "../../application/queryKeys";
import { Button } from "../../components/ui/Button";
import type { BackupRecord, BackupVerification } from "../../domain/models";
import { desktop } from "../../infrastructure/tauri/commands";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";

export function BackupsSection() {
  const queryClient = useQueryClient();
  const [label, setLabel] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [verifications, setVerifications] = useState<Record<string, BackupVerification>>({});

  const backupsQuery = useQuery({
    queryKey: queryKeys.backups,
    queryFn: desktop.backupList,
  });
  const backups = backupsQuery.data ?? [];

  const invalidateRelated = () => {
    void queryClient.invalidateQueries({ queryKey: queryKeys.backups });
    void queryClient.invalidateQueries({ queryKey: queryKeys.diagnostics });
    void queryClient.invalidateQueries({ queryKey: queryKeys.privacyDashboard });
  };

  const createMutation = useMutation({
    mutationFn: () => desktop.backupCreate(label.trim() ? label.trim() : null),
    onSuccess: (backup) => {
      setLabel("");
      setError(null);
      setMessage(backupCreatedMessage(backup));
      invalidateRelated();
    },
    onError: (mutationError) => {
      setError(errorMessage(mutationError));
      setMessage(null);
    },
  });

  const verifyMutation = useMutation({
    mutationFn: (id: string) => desktop.backupVerify(id),
    onSuccess: (result, id) => {
      setVerifications((current) => ({ ...current, [id]: result }));
      setError(null);
      setMessage(backupVerificationMessage(result));
    },
    onError: (mutationError) => {
      setError(errorMessage(mutationError));
      setMessage(null);
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (backup: BackupRecord) => desktop.backupDelete(backup.id),
    onSuccess: (_, backup) => {
      setError(null);
      setMessage(`Deleted backup "${backup.label}".`);
      invalidateRelated();
    },
    onError: (mutationError) => {
      setError(errorMessage(mutationError));
      setMessage(null);
    },
  });

  const restoreMutation = useMutation({
    mutationFn: (backup: BackupRecord) => desktop.backupRestore(backup.id),
    onSuccess: (outcome) => {
      setError(null);
      setMessage(backupRestoredMessage(outcome));
      invalidateRelated();
    },
    onError: (mutationError) => {
      setError(errorMessage(mutationError));
      setMessage(null);
    },
  });

  const requestDelete = (backup: BackupRecord) => {
    if (!window.confirm(backupDeleteMessage(backup))) return;
    deleteMutation.mutate(backup);
  };

  const requestRestore = (backup: BackupRecord) => {
    if (!window.confirm(backupRestoreMessage(backup))) return;
    restoreMutation.mutate(backup);
  };

  return (
    <div>
      <p className="bergag-eyebrow">Backups</p>
      <h2 className="bergag-title bergag-title--small">Verified local backups</h2>
      <p className="bergag-subtitle">
        Bergag writes a consistent SQLite snapshot into its own local backup directory, records a
        manifest with a SHA-256 checksum, and verifies checksum, profile list, and database
        integrity before any restore. Credentials stay in Windows Credential Manager and are never
        part of a backup. The five newest automatic backups are retained; manual backups remain
        until you delete them.
      </p>

      <div className="bergag-card bergag-card--padded" style={{ marginTop: 18 }}>
        <div className="bergag-field">
          <label className="bergag-field__label" htmlFor="backup-label">
            Backup label (optional)
          </label>
          <input
            id="backup-label"
            className="bergag-input"
            value={label}
            placeholder={backupLabelOrDefault("")}
            onChange={(event) => setLabel(event.target.value)}
          />
        </div>
        <div className="bergag-inline-actions">
          <Button
            size="sm"
            variant="primary"
            disabled={createMutation.isPending}
            aria-busy={createMutation.isPending}
            onClick={() => createMutation.mutate()}
          >
            Create verified backup
          </Button>
        </div>
        <div className="bergag-field__hint" role="status" aria-live="polite">
          {createMutation.isPending
            ? "Creating a verified local backup…"
            : (message ?? "Creating a backup does not change or interrupt your current library.")}
        </div>
        {error ? (
          <div className="bergag-error" role="alert">
            {error}
          </div>
        ) : null}
      </div>

      <h3 className="bergag-section-title">Stored backups</h3>
      {backupsQuery.isLoading ? <div className="bergag-loading">Loading backups…</div> : null}
      {backupsQuery.isError ? (
        <div className="bergag-error" role="alert">
          {errorMessage(backupsQuery.error)}
        </div>
      ) : null}
      {backups.length === 0 && !backupsQuery.isLoading ? (
        <div className="bergag-note">No backups have been created in this installation.</div>
      ) : null}
      <div className="bergag-ai-provider-list">
        {backups.map((backup) => {
          const verification = verifications[backup.id];
          const backupName = `${backup.label} (${formatBackupDate(backup.created_at)})`;
          return (
            <div className="bergag-ai-provider" key={backup.id}>
              <div className="bergag-ai-provider__head">
                <div>
                  <div className="bergag-stat__label">{backup.label}</div>
                  <div className="bergag-stat__detail">
                    {formatBackupDate(backup.created_at)} · schema {backup.schema_version} ·{" "}
                    {backup.profile_count} profile{backup.profile_count === 1 ? "" : "s"} ·{" "}
                    {formatBackupSize(backup.size_bytes)}
                  </div>
                  {verification ? (
                    <div className="bergag-field__hint" role="status" aria-live="polite">
                      {backupVerificationMessage(verification)}
                    </div>
                  ) : null}
                </div>
                <div className="bergag-inline-actions">
                  {backup.automatic ? (
                    <span className="bergag-badge">Automatic</span>
                  ) : (
                    <span className="bergag-badge bergag-badge--accent">Manual</span>
                  )}
                </div>
              </div>
              <div className="bergag-inline-actions">
                <Button
                  size="sm"
                  disabled={verifyMutation.isPending}
                  aria-label={`Verify backup ${backupName}`}
                  onClick={() => verifyMutation.mutate(backup.id)}
                >
                  Verify
                </Button>
                <Button
                  size="sm"
                  variant="danger"
                  disabled={deleteMutation.isPending || restoreMutation.isPending}
                  aria-label={`Delete backup ${backupName}`}
                  onClick={() => requestDelete(backup)}
                >
                  Delete
                </Button>
                <Button
                  size="sm"
                  variant="primary"
                  disabled={restoreMutation.isPending || deleteMutation.isPending}
                  aria-busy={restoreMutation.isPending}
                  aria-label={`Restore backup ${backupName}`}
                  onClick={() => requestRestore(backup)}
                >
                  Restore
                </Button>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
