import { errorMessage } from "../../application/articleService";
import { queryKeys } from "../../application/queryKeys";
import { desktop } from "../../infrastructure/tauri/commands";
import { useQuery } from "@tanstack/react-query";

export function SecuritySection() {
  const statusQuery = useQuery({
    queryKey: queryKeys.desktopSecurity,
    queryFn: desktop.desktopSecurityStatus,
  });
  const status = statusQuery.data;

  return (
    <div>
      <p className="bergag-eyebrow">Security</p>
      <h2 className="bergag-title bergag-title--small">Local security posture</h2>
      <p className="bergag-subtitle">
        This page reports the security posture compiled into this installation. Command-surface and
        redaction checks are build-time assertions covered by tests, not live scans. It does not
        imply encryption or a profile lock that does not exist.
      </p>

      {statusQuery.isLoading ? (
        <div className="bergag-loading">Reading local security state…</div>
      ) : null}
      {statusQuery.isError ? (
        <div className="bergag-error" role="alert">
          {errorMessage(statusQuery.error)}
        </div>
      ) : null}

      {status ? (
        <>
          <div className="bergag-card bergag-card--padded" style={{ marginTop: 18 }}>
            <div className="bergag-switch-row">
              <div className="bergag-switch-row__text">
                <span className="bergag-switch-row__label">Profile lock</span>
                <span className="bergag-switch-row__hint">
                  {status.lockDetail} {status.lockedProfileCount} profile(s) carry the dormant flag.
                </span>
              </div>
              <span className="bergag-badge">Unavailable</span>
            </div>
            <div className="bergag-switch-row">
              <div className="bergag-switch-row__text">
                <span className="bergag-switch-row__label">Credential storage</span>
                <span className="bergag-switch-row__hint">{status.credentialStore}</span>
              </div>
              <span className="bergag-badge">Local</span>
            </div>
            <div className="bergag-switch-row">
              <div className="bergag-switch-row__text">
                <span className="bergag-switch-row__label">Credential read command</span>
                <span className="bergag-switch-row__hint">
                  {status.credentialGetterCommand
                    ? "A command exposes stored credentials."
                    : "No command reads a saved key back into the UI or API."}
                </span>
              </div>
              <span className="bergag-badge">Off</span>
            </div>
            <div className="bergag-switch-row">
              <div className="bergag-switch-row__text">
                <span className="bergag-switch-row__label">Secret columns in SQLite</span>
                <span className="bergag-switch-row__hint">
                  {status.sqliteSecretColumns === 0
                    ? "No API key, password, token, or secret column exists in the local database."
                    : `${status.sqliteSecretColumns} secret-like column(s) were found in the local database.`}
                </span>
              </div>
              <span className="bergag-badge">{status.sqliteSecretColumns}</span>
            </div>
          </div>

          <div className="bergag-card bergag-card--padded">
            <div className="bergag-switch-row">
              <div className="bergag-switch-row__text">
                <span className="bergag-switch-row__label">Public URL validation</span>
                <span className="bergag-switch-row__hint">
                  {status.publicUrlValidation
                    ? "Network commands reject private and unsafe destinations before a request is made."
                    : "Public URL validation is not active in this build."}
                </span>
              </div>
              <span className="bergag-badge">
                {status.publicUrlValidation ? "Enforced" : "Off"}
              </span>
            </div>
            <div className="bergag-switch-row">
              <div className="bergag-switch-row__text">
                <span className="bergag-switch-row__label">HTML sanitization</span>
                <span className="bergag-switch-row__hint">
                  {status.htmlSanitization
                    ? "Fetched article content is sanitized before it is stored or displayed."
                    : "HTML sanitization is not active in this build."}
                </span>
              </div>
              <span className="bergag-badge">{status.htmlSanitization ? "Enforced" : "Off"}</span>
            </div>
            <div className="bergag-switch-row">
              <div className="bergag-switch-row__text">
                <span className="bergag-switch-row__label">Log redaction</span>
                <span className="bergag-switch-row__hint">
                  {status.logRedaction
                    ? "Secrets in diagnostic errors are replaced before they reach local logs."
                    : "Log redaction is not active in this build."}
                </span>
              </div>
              <span className="bergag-badge">{status.logRedaction ? "Enforced" : "Off"}</span>
            </div>
            <div className="bergag-switch-row">
              <div className="bergag-switch-row__text">
                <span className="bergag-switch-row__label">
                  Generic shell or filesystem commands
                </span>
                <span className="bergag-switch-row__hint">
                  {status.genericShellOrFilesystemAccess
                    ? "An unrestricted execute or file access command is present in this build."
                    : "The Tauri command surface is narrow; there is no unrestricted execute or file access command."}
                </span>
              </div>
              <span className="bergag-badge">
                {status.genericShellOrFilesystemAccess ? "Present" : "None"}
              </span>
            </div>
          </div>

          <div className="bergag-note" style={{ marginTop: 16 }}>
            Profile lock is shown as unavailable because the dormant is_locked and lock_hint fields
            do not encrypt or restrict access. No control on this page changes those fields.
          </div>
        </>
      ) : null}
    </div>
  );
}
