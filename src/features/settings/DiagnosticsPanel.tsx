import { useQuery } from "@tanstack/react-query";
import { RefreshCw } from "lucide-react";
import { queryKeys } from "../../application/queryKeys";
import { Button } from "../../components/ui/Button";
import { desktop } from "../../infrastructure/tauri/commands";

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export function DiagnosticsPanel() {
  const diagnostics = useQuery({
    queryKey: queryKeys.diagnostics,
    queryFn: desktop.getDiagnostics,
  });
  const logs = useQuery({
    queryKey: queryKeys.recentLogs,
    queryFn: () => desktop.getRecentLogs(50),
  });

  const refresh = () => {
    void diagnostics.refetch();
    void logs.refetch();
  };

  if (diagnostics.isLoading) return <div className="bergag-loading">Reading diagnostics…</div>;
  if (diagnostics.error) {
    return (
      <div className="bergag-error" role="alert">
        {diagnostics.error instanceof Error ? diagnostics.error.message : String(diagnostics.error)}
      </div>
    );
  }
  if (!diagnostics.data) return null;

  const data = diagnostics.data;
  return (
    <div>
      <div className="bergag-inline-actions" style={{ justifyContent: "flex-end" }}>
        <Button
          size="sm"
          icon={<RefreshCw size={13} aria-hidden="true" />}
          onClick={refresh}
          disabled={diagnostics.isFetching || logs.isFetching}
        >
          Refresh
        </Button>
      </div>

      <div className="bergag-card-grid">
        <div className="bergag-card bergag-card--padded">
          <div className="bergag-stat__label">Bergag version</div>
          <div className="bergag-stat__value">{data.app_version}</div>
          <div className="bergag-stat__detail">Tauri runtime {data.tauri_version}</div>
        </div>
        <div className="bergag-card bergag-card--padded">
          <div className="bergag-stat__label">Database integrity</div>
          <div className="bergag-stat__value">{data.database.integrity_check}</div>
          <div className="bergag-stat__detail">
            {data.os} · {data.arch}
          </div>
        </div>
        <div className="bergag-card bergag-card--padded">
          <div className="bergag-stat__label">SQLite</div>
          <div className="bergag-stat__value">{data.database.sqlite_version}</div>
          <div className="bergag-stat__detail">
            FTS5: {data.database.fts5_available ? "available" : "unavailable"}
          </div>
        </div>
        <div className="bergag-card bergag-card--padded">
          <div className="bergag-stat__label">Database size</div>
          <div className="bergag-stat__value">{formatBytes(data.database.size_bytes)}</div>
          <div className="bergag-stat__detail">Migration {data.database.user_version}</div>
        </div>
      </div>

      <h3 className="bergag-section-title">Database location</h3>
      <div className="bergag-code-block">{data.database.path}</div>

      <h3 className="bergag-section-title">Recent redacted log lines</h3>
      {logs.isLoading ? <div className="bergag-loading">Reading logs…</div> : null}
      {logs.error ? (
        <div className="bergag-error" role="alert">
          {logs.error instanceof Error ? logs.error.message : String(logs.error)}
        </div>
      ) : null}
      <div className="bergag-code-block">
        {logs.data && logs.data.length > 0 ? logs.data.join("\n") : "No log entries yet."}
      </div>
    </div>
  );
}
