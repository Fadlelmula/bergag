import { useQuery } from "@tanstack/react-query";
import { errorMessage, formatBytes } from "../../application/articleService";
import { countLabel, formatCount } from "../../application/analyticsService";
import { queryKeys } from "../../application/queryKeys";
import { desktop } from "../../infrastructure/tauri/commands";

export function StorageSection() {
  const stats = useQuery({
    queryKey: queryKeys.storageStats("settings"),
    queryFn: desktop.analyticsStorage,
  });

  return (
    <div>
      <p className="bergag-eyebrow">Storage</p>
      <h2 className="bergag-title bergag-title--small">Local disk usage</h2>
      <p className="bergag-subtitle">
        Bergag keeps its database and cached content on this device. These counts come from SQLite;
        no cleanup or deletion happens here.
      </p>

      {stats.isLoading ? <div className="bergag-loading">Loading storage statistics…</div> : null}
      {stats.isError ? (
        <div className="bergag-error" role="alert">
          {errorMessage(stats.error)}
        </div>
      ) : null}
      {stats.data ? (
        <div className="bergag-card bergag-card--padded" style={{ marginTop: 18 }}>
          <div className="bergag-switch-row">
            <div className="bergag-switch-row__text">
              <span className="bergag-switch-row__label">Database file</span>
              <span className="bergag-switch-row__hint">
                Whole local SQLite file, shared by profiles.
              </span>
            </div>
            <span className="bergag-badge">{formatBytes(stats.data.database_bytes)}</span>
          </div>
          <div className="bergag-switch-row">
            <div className="bergag-switch-row__text">
              <span className="bergag-switch-row__label">Cached images</span>
              <span className="bergag-switch-row__hint">Offline images owned by this profile.</span>
            </div>
            <span className="bergag-badge">
              {countLabel(stats.data.cached_images, "image")} ·{" "}
              {formatBytes(stats.data.cached_image_bytes)}
            </span>
          </div>
          <div className="bergag-switch-row">
            <div className="bergag-switch-row__text">
              <span className="bergag-switch-row__label">Extracted articles</span>
              <span className="bergag-switch-row__hint">
                Articles with locally stored readable content.
              </span>
            </div>
            <span className="bergag-badge">{formatCount(stats.data.extracted_articles)}</span>
          </div>
          <div className="bergag-switch-row">
            <div className="bergag-switch-row__text">
              <span className="bergag-switch-row__label">Stored text</span>
              <span className="bergag-switch-row__hint">
                Characters in extracted article content.
              </span>
            </div>
            <span className="bergag-badge">{formatCount(stats.data.text_characters)}</span>
          </div>
        </div>
      ) : null}

      <div className="bergag-note" style={{ marginTop: 16 }}>
        To remove cached article content, use Settings → Offline. Any such action is explicit and
        never changes your feeds, labels, or reading history.
      </div>
    </div>
  );
}
