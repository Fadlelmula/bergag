import { useQuery } from "@tanstack/react-query";
import { useProfile } from "../../app/ProfileProvider";
import { queryKeys } from "../../application/queryKeys";
import { desktop } from "../../infrastructure/tauri/commands";

export function SearchSection() {
  const { activeProfile } = useProfile();
  const stats = useQuery({
    queryKey: queryKeys.searchStats(activeProfile?.id ?? "none"),
    queryFn: desktop.searchStats,
  });

  const value = (count: number | undefined) => (stats.isLoading ? "…" : (count ?? 0));

  return (
    <div>
      <p className="bergag-eyebrow">Search</p>
      <h2 className="bergag-title bergag-title--small">Local search & stories</h2>
      <p className="bergag-subtitle">
        Bergag indexes stored article titles, text, authors, and feed titles in a local SQLite FTS5
        table. Queries never leave this device.
      </p>

      <div className="bergag-card bergag-card--padded" style={{ marginTop: 18 }}>
        <div className="bergag-switch-row">
          <div className="bergag-switch-row__text">
            <span className="bergag-switch-row__label">Indexed documents</span>
            <span className="bergag-switch-row__hint">
              Articles with a local full-text entry in this profile.
            </span>
          </div>
          <span className="bergag-badge">{value(stats.data?.indexed_documents)}</span>
        </div>
        <div className="bergag-switch-row">
          <div className="bergag-switch-row__text">
            <span className="bergag-switch-row__label">Saved searches</span>
            <span className="bergag-switch-row__hint">
              Queries and scopes you chose to keep for later.
            </span>
          </div>
          <span className="bergag-badge">{value(stats.data?.presets)}</span>
        </div>
        <div className="bergag-switch-row">
          <div className="bergag-switch-row__text">
            <span className="bergag-switch-row__label">Story groups</span>
            <span className="bergag-switch-row__hint">
              Local groups created by duplicate detection.
            </span>
          </div>
          <span className="bergag-badge">{value(stats.data?.story_groups)}</span>
        </div>
        <div className="bergag-switch-row">
          <div className="bergag-switch-row__text">
            <span className="bergag-switch-row__label">Grouped articles</span>
            <span className="bergag-switch-row__hint">
              Articles linked into a story group; originals are never removed.
            </span>
          </div>
          <span className="bergag-badge">{value(stats.data?.story_items)}</span>
        </div>
        <div className="bergag-switch-row">
          <div className="bergag-switch-row__text">
            <span className="bergag-switch-row__label">Semantic search</span>
            <span className="bergag-switch-row__hint">
              Planned for a later milestone. No model runs and no content is uploaded.
            </span>
          </div>
          <span className="bergag-badge bergag-badge--accent">Off</span>
        </div>
      </div>

      <div className="bergag-note" style={{ marginTop: 16 }}>
        Search, saved searches, duplicate detection, and story timelines are available from the
        Search view. Duplicate detection only runs when you choose Detect duplicates.
      </div>
    </div>
  );
}
