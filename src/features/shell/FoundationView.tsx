import { Button } from "../../components/ui/Button";
import type { Diagnostics, Profile } from "../../domain/models";

interface FoundationViewProps {
  activeProfile: Profile | undefined;
  profileCount: number;
  diagnostics: Diagnostics | undefined;
  diagnosticsLoading: boolean;
  diagnosticsError: string | null;
  onOpenSettings: () => void;
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export function FoundationView({
  activeProfile,
  profileCount,
  diagnostics,
  diagnosticsLoading,
  diagnosticsError,
  onOpenSettings,
}: FoundationViewProps) {
  return (
    <div className="bergag-content bergag-content--narrow">
      <header className="bergag-content-header">
        <p className="bergag-eyebrow">Bergag · Local workspace</p>
        <h1 className="bergag-title">Your local intelligence cockpit is initialized.</h1>
        <p className="bergag-subtitle">
          Bergag is running from a private local database. Feed subscriptions, the article reader,
          and the local discovery directory and rules are available from the sidebar. Search and
          later capabilities remain clearly labeled as planned.
        </p>
      </header>

      {diagnosticsLoading ? <div className="bergag-loading">Reading local diagnostics…</div> : null}
      {diagnosticsError ? (
        <div className="bergag-error" role="alert">
          Diagnostics unavailable: {diagnosticsError}
        </div>
      ) : null}

      {diagnostics ? (
        <div className="bergag-card-grid">
          <div className="bergag-card bergag-card--padded">
            <div className="bergag-stat__label">Active profile</div>
            <div className="bergag-stat__value">{activeProfile?.name ?? "—"}</div>
            <div className="bergag-stat__detail">
              {profileCount} profile{profileCount === 1 ? "" : "s"} stored locally
            </div>
          </div>
          <div className="bergag-card bergag-card--padded">
            <div className="bergag-stat__label">Database integrity</div>
            <div className="bergag-stat__value">{diagnostics.database.integrity_check}</div>
            <div className="bergag-stat__detail">
              Migration {diagnostics.database.user_version} · {diagnostics.database.migration_count}{" "}
              applied
            </div>
          </div>
          <div className="bergag-card bergag-card--padded">
            <div className="bergag-stat__label">SQLite</div>
            <div className="bergag-stat__value">{diagnostics.database.sqlite_version}</div>
            <div className="bergag-stat__detail">
              FTS5{" "}
              {diagnostics.database.fts5_available ? "available for later search" : "unavailable"}
            </div>
          </div>
          <div className="bergag-card bergag-card--padded">
            <div className="bergag-stat__label">Database size</div>
            <div className="bergag-stat__value">{formatBytes(diagnostics.database.size_bytes)}</div>
            <div className="bergag-stat__detail">
              {diagnostics.os} · {diagnostics.arch}
            </div>
          </div>
        </div>
      ) : null}

      <div className="bergag-card bergag-card--padded">
        <h2 className="bergag-title bergag-title--small">Privacy posture</h2>
        <p className="bergag-subtitle">
          Telemetry is off and cannot be enabled from the interface. No AI provider is configured.
          Article content, labels, rules, and the discovery directory stay in this profile. Nothing
          is sent to a remote service by Bergag core.
        </p>
        <div style={{ marginTop: 14 }}>
          <Button variant="secondary" onClick={onOpenSettings}>
            Open privacy and diagnostics
          </Button>
        </div>
      </div>

      <h2 className="bergag-section-title">Available now</h2>
      <ul className="bergag-bullet-list">
        <li>RSS, Atom, and JSON subscriptions with folders, health, and OPML</li>
        <li>Sanitized article reading, state, notes, highlights, and collections</li>
        <li>Local discovery directory with feed preview, categories, and trust levels</li>
        <li>Tags, topics, local rules, smart feeds, and rule run history</li>
      </ul>
    </div>
  );
}
