import { useProfile } from "../../app/ProfileProvider";
import { errorMessage, formatBytes } from "../../application/articleService";
import { queryKeys } from "../../application/queryKeys";
import { Button } from "../../components/ui/Button";
import { SwitchField } from "../../components/ui/SwitchField";
import type { OfflineSettings } from "../../domain/models";
import { normalizeOffline, SETTINGS_KEYS } from "../../domain/settings";
import { desktop } from "../../infrastructure/tauri/commands";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";

export function OfflineSection() {
  const { activeProfile } = useProfile();
  const profileId = activeProfile?.id ?? "";
  const queryClient = useQueryClient();
  const [error, setError] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);

  const settings = useQuery({
    queryKey: queryKeys.settings(profileId, SETTINGS_KEYS.offline),
    queryFn: () => desktop.getProfileSetting<OfflineSettings>(SETTINGS_KEYS.offline),
    enabled: Boolean(profileId),
  });
  const stats = useQuery({
    queryKey: queryKeys.articleCacheStats(profileId),
    queryFn: desktop.articleCacheStats,
    enabled: Boolean(profileId),
  });
  const offline = normalizeOffline(settings.data);

  const save = useMutation({
    mutationFn: (cacheImages: boolean) =>
      desktop.setProfileSetting(SETTINGS_KEYS.offline, { ...offline, cacheImages }),
    onSuccess: () =>
      queryClient.invalidateQueries({
        queryKey: queryKeys.settings(profileId, SETTINGS_KEYS.offline),
      }),
    onError: (mutationError) => setError(errorMessage(mutationError)),
  });

  const clear = useMutation({
    mutationFn: () => desktop.clearArticleCache(null),
    onSuccess: (removed) => {
      setMessage(`Removed cached content for ${removed} article${removed === 1 ? "" : "s"}.`);
      setError(null);
      void queryClient.invalidateQueries({ queryKey: queryKeys.articles(profileId) });
      void queryClient.invalidateQueries({ queryKey: queryKeys.articleCacheStats(profileId) });
    },
    onError: (mutationError) => setError(errorMessage(mutationError)),
  });

  const requestClear = () => {
    const confirmed = window.confirm(
      "Remove cached article text and images for this profile? Notes, highlights, read state, collections, and feed summaries are kept.",
    );
    if (!confirmed) return;
    setMessage(null);
    clear.mutate();
  };

  return (
    <div>
      <p className="bergag-eyebrow">Offline</p>
      <h2 className="bergag-title bergag-title--small">Local reading cache</h2>
      <p className="bergag-subtitle">
        Article text is always stored locally. Images are embedded only when you explicitly extract
        an article and image caching is enabled.
      </p>

      <div className="bergag-card bergag-card--padded" style={{ marginTop: 18 }}>
        <SwitchField
          label="Cache article images"
          hint="Stores supported raster images as local data URLs when extracting a full article."
          checked={offline.cacheImages}
          disabled={save.isPending}
          onCheckedChange={(checked) => save.mutate(checked)}
        />
      </div>

      <h3 className="bergag-section-title">Current profile usage</h3>
      {stats.isLoading ? <div className="bergag-loading">Reading cache statistics…</div> : null}
      {stats.error ? (
        <div className="bergag-error" role="alert">
          {errorMessage(stats.error)}
        </div>
      ) : null}
      {stats.data ? (
        <div className="bergag-card-grid">
          <div className="bergag-card bergag-card--padded">
            <div className="bergag-stat__label">Cached articles</div>
            <div className="bergag-stat__value">{stats.data.articles}</div>
          </div>
          <div className="bergag-card bergag-card--padded">
            <div className="bergag-stat__label">Cached images</div>
            <div className="bergag-stat__value">{stats.data.images}</div>
          </div>
          <div className="bergag-card bergag-card--padded">
            <div className="bergag-stat__label">Local storage</div>
            <div className="bergag-stat__value">
              {formatBytes(stats.data.article_bytes + stats.data.image_bytes)}
            </div>
          </div>
        </div>
      ) : null}

      <div className="bergag-inline-actions" style={{ marginTop: 8 }}>
        <Button variant="danger" size="sm" disabled={clear.isPending} onClick={requestClear}>
          {clear.isPending ? "Clearing…" : "Clear cached article content"}
        </Button>
        <span className="bergag-field__hint">
          Feed summaries, notes, highlights, collections, and reading state are preserved.
        </span>
      </div>

      {message ? (
        <div className="bergag-note" style={{ marginTop: 14 }}>
          {message}
        </div>
      ) : null}
      {error ? (
        <p className="bergag-error" role="alert" style={{ marginTop: 14 }}>
          {error}
        </p>
      ) : null}
    </div>
  );
}
