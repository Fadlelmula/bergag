import { errorMessage } from "../../application/articleService";
import { queryKeys } from "../../application/queryKeys";
import { desktop } from "../../infrastructure/tauri/commands";
import { useQuery } from "@tanstack/react-query";

export function PrivacySection() {
  const dashboard = useQuery({
    queryKey: queryKeys.privacyDashboard,
    queryFn: desktop.privacyDashboard,
  });
  const data = dashboard.data;

  return (
    <div>
      <p className="bergag-eyebrow">Privacy</p>
      <h2 className="bergag-title bergag-title--small">Privacy dashboard</h2>
      <p className="bergag-subtitle">
        Counts and paths are read from this installation when the page opens. Posture flags are
        build-time assertions covered by tests. Bergag has no account, telemetry, hidden analytics,
        or background AI. Nothing here is a demo value.
      </p>

      {dashboard.isLoading ? (
        <div className="bergag-loading">Reading local privacy state…</div>
      ) : null}
      {dashboard.isError ? (
        <div className="bergag-error" role="alert">
          {errorMessage(dashboard.error)}
        </div>
      ) : null}

      {data ? (
        <>
          <div className="bergag-card bergag-card--padded" style={{ marginTop: 18 }}>
            <div className="bergag-switch-row">
              <div className="bergag-switch-row__text">
                <span className="bergag-switch-row__label">Telemetry</span>
                <span className="bergag-switch-row__hint">
                  {data.telemetry_enabled
                    ? "Telemetry is enabled in this build."
                    : "No telemetry service is configured or available."}
                </span>
              </div>
              <span className="bergag-badge bergag-badge--accent">
                {data.telemetry_enabled ? "On" : "Off"}
              </span>
            </div>
            <div className="bergag-switch-row">
              <div className="bergag-switch-row__text">
                <span className="bergag-switch-row__label">AI processing</span>
                <span className="bergag-switch-row__hint">
                  {data.ai_enabled
                    ? `Opted in · ${data.remote_ai_provider_count} remote and ${data.local_ai_provider_count} local provider(s) configured. Explicit actions only.`
                    : "No AI provider is enabled for this profile."}
                </span>
              </div>
              <span className="bergag-badge">{data.ai_enabled ? "Opt-in" : "Off"}</span>
            </div>
            <div className="bergag-switch-row">
              <div className="bergag-switch-row__text">
                <span className="bergag-switch-row__label">Database integrity</span>
                <span className="bergag-switch-row__hint">
                  Schema {data.schema_version} · {data.integrity_check}
                </span>
              </div>
              <span className="bergag-badge">{data.integrity_check}</span>
            </div>
          </div>

          <h3 className="bergag-section-title">Local data inventory</h3>
          <div className="bergag-card bergag-card--padded">
            <div className="bergag-card-grid">
              <div className="bergag-stat bergag-card bergag-card--padded">
                <div className="bergag-stat__label">Profiles</div>
                <div className="bergag-stat__value">{data.profile_count}</div>
              </div>
              <div className="bergag-stat bergag-card bergag-card--padded">
                <div className="bergag-stat__label">Feeds</div>
                <div className="bergag-stat__value">{data.feed_count}</div>
              </div>
              <div className="bergag-stat bergag-card bergag-card--padded">
                <div className="bergag-stat__label">Articles</div>
                <div className="bergag-stat__value">{data.article_count}</div>
              </div>
              <div className="bergag-stat bergag-card bergag-card--padded">
                <div className="bergag-stat__label">Notes</div>
                <div className="bergag-stat__value">{data.note_count}</div>
              </div>
              <div className="bergag-stat bergag-card bergag-card--padded">
                <div className="bergag-stat__label">Highlights</div>
                <div className="bergag-stat__value">{data.highlight_count}</div>
              </div>
              <div className="bergag-stat bergag-card bergag-card--padded">
                <div className="bergag-stat__label">Labels</div>
                <div className="bergag-stat__value">{data.label_count}</div>
              </div>
              <div className="bergag-stat bergag-card bergag-card--padded">
                <div className="bergag-stat__label">Rules</div>
                <div className="bergag-stat__value">{data.rule_count}</div>
              </div>
              <div className="bergag-stat bergag-card bergag-card--padded">
                <div className="bergag-stat__label">Dashboards</div>
                <div className="bergag-stat__value">{data.dashboard_count}</div>
              </div>
              <div className="bergag-stat bergag-card bergag-card--padded">
                <div className="bergag-stat__label">Integrations</div>
                <div className="bergag-stat__value">{data.integration_source_count}</div>
              </div>
              <div className="bergag-stat bergag-card bergag-card--padded">
                <div className="bergag-stat__label">AI requests</div>
                <div className="bergag-stat__value">{data.ai_usage_event_count}</div>
              </div>
              <div className="bergag-stat bergag-card bergag-card--padded">
                <div className="bergag-stat__label">Backups</div>
                <div className="bergag-stat__value">{data.backup_count}</div>
              </div>
            </div>
          </div>

          <h3 className="bergag-section-title">Network surfaces</h3>
          <div className="bergag-card bergag-card--padded">
            <div className="bergag-switch-row">
              <div className="bergag-switch-row__text">
                <span className="bergag-switch-row__label">Public feeds</span>
                <span className="bergag-switch-row__hint">
                  Fetched for active subscriptions on a 60-second local schedule, or when you
                  preview or refresh manually.
                </span>
              </div>
              <span className="bergag-badge">{data.feed_count} subscribed</span>
            </div>
            <div className="bergag-switch-row">
              <div className="bergag-switch-row__text">
                <span className="bergag-switch-row__label">Integrations</span>
                <span className="bergag-switch-row__hint">
                  Public, credential-free feeds only. Bluesky and website monitoring remain planned.
                </span>
              </div>
              <span className="bergag-badge">{data.integration_source_count} connected</span>
            </div>
            <div className="bergag-switch-row">
              <div className="bergag-switch-row__text">
                <span className="bergag-switch-row__label">AI providers</span>
                <span className="bergag-switch-row__hint">
                  No request is sent without opt-in and per-action confirmation. Keys stay in
                  Windows Credential Manager.
                </span>
              </div>
              <span className="bergag-badge">{data.ai_provider_count} configured</span>
            </div>
          </div>

          <h3 className="bergag-section-title">Local storage locations</h3>
          <div className="bergag-card bergag-card--padded">
            <div className="bergag-field__label">Database</div>
            <div className="bergag-stat__detail">{data.database_path}</div>
            <div className="bergag-field__label" style={{ marginTop: 12 }}>
              Backups
            </div>
            <div className="bergag-stat__detail">{data.backup_directory}</div>
            <div className="bergag-field__label" style={{ marginTop: 12 }}>
              Logs
            </div>
            <div className="bergag-stat__detail">{data.log_directory}</div>
          </div>
        </>
      ) : null}

      <div className="bergag-note" style={{ marginTop: 16 }}>
        This dashboard reports state only. To change AI access, use Settings → AI. To create,
        verify, restore, or delete local backups, use Settings → Backups. Removing an integration or
        a backup never changes saved articles elsewhere in the library.
      </div>
    </div>
  );
}
