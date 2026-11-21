import { useProfile } from "../../app/ProfileProvider";
import { errorMessage } from "../../application/articleService";
import {
  availableIntegrationPlatforms,
  integrationDeleteMessage,
  integrationKindLabel,
  integrationPlatform,
  integrationPreviewIsCurrent,
  integrationPreviewNotices,
  integrationRemoveMessage,
  plannedIntegrationPlatforms,
  validateIntegrationSelection,
  type IntegrationPreviewState,
} from "../../application/integrationService";
import { queryKeys } from "../../application/queryKeys";
import { Button } from "../../components/ui/Button";
import type { IntegrationKind, IntegrationSource, TrustLevel } from "../../domain/models";
import { desktop } from "../../infrastructure/tauri/commands";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";

const TRUST_LEVELS: TrustLevel[] = ["untrusted", "low", "medium", "high"];

export function IntegrationsSection() {
  const { activeProfile } = useProfile();
  const profileId = activeProfile?.id ?? "";
  const queryClient = useQueryClient();
  const [selectedKind, setSelectedKind] = useState<IntegrationKind | "">("");
  const [input, setInput] = useState("");
  const [category, setCategory] = useState("");
  const [trustLevel, setTrustLevel] = useState<TrustLevel>("untrusted");
  const [preview, setPreview] = useState<IntegrationPreviewState | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);

  const platformsQuery = useQuery({
    queryKey: queryKeys.integrationPlatforms,
    queryFn: desktop.integrationPlatforms,
  });
  const sourcesQuery = useQuery({
    queryKey: queryKeys.integrationSources(profileId),
    queryFn: desktop.integrationList,
    enabled: Boolean(profileId),
  });

  const platforms = platformsQuery.data ?? [];
  const available = availableIntegrationPlatforms(platforms);
  const planned = plannedIntegrationPlatforms(platforms);
  const kind =
    selectedKind || available.find((platform) => platform.available)?.kind || ("" as const);
  const selected = kind ? integrationPlatform(platforms, kind as IntegrationKind) : null;
  const sources = sourcesQuery.data ?? [];
  const validationError = validateIntegrationSelection(
    kind as IntegrationKind | "",
    input,
    platforms,
  );
  const previewMatches = integrationPreviewIsCurrent(preview, kind as IntegrationKind, input);

  const invalidateRelated = () => {
    void queryClient.invalidateQueries({ queryKey: queryKeys.integrationSources(profileId) });
    void queryClient.invalidateQueries({ queryKey: queryKeys.feeds(profileId) });
    void queryClient.invalidateQueries({ queryKey: queryKeys.libraryStats(profileId) });
    void queryClient.invalidateQueries({ queryKey: queryKeys.feedHealth(profileId, 20) });
  };

  const previewMutation = useMutation({
    mutationFn: ({ platform, value }: { platform: IntegrationKind; value: string }) =>
      desktop.integrationPreview(platform, value.trim()),
    onSuccess: (result, variables) => {
      setPreview({ kind: variables.platform, input: variables.value.trim(), result });
      setError(null);
      setMessage(null);
    },
    onError: (mutationError) => {
      setPreview(null);
      setError(errorMessage(mutationError));
      setMessage(null);
    },
  });

  const addMutation = useMutation({
    mutationFn: () => {
      if (!previewMatches || !preview) {
        throw new Error("Preview this source before adding it.");
      }
      return desktop.integrationAdd(
        preview.kind,
        preview.input,
        category.trim() || null,
        trustLevel,
        null,
      );
    },
    onSuccess: (source) => {
      setInput("");
      setCategory("");
      setTrustLevel("untrusted");
      setPreview(null);
      setError(null);
      setMessage(`${source.display_name} connected and subscribed to its public feed.`);
      invalidateRelated();
    },
    onError: (mutationError) => {
      setError(errorMessage(mutationError));
      setMessage(null);
    },
  });

  const trustMutation = useMutation({
    mutationFn: ({ sourceId, level }: { sourceId: string; level: TrustLevel }) =>
      desktop.integrationSetTrust(sourceId, level),
    onSuccess: (source) => {
      setError(null);
      setMessage(`${source.display_name} trust set to ${source.trust_level}.`);
      invalidateRelated();
    },
    onError: (mutationError) => setError(errorMessage(mutationError)),
  });

  const refreshMutation = useMutation({
    mutationFn: (sourceId: string) => desktop.integrationRefresh(sourceId),
    onSuccess: (outcomes) => {
      const added = outcomes.reduce((total, outcome) => total + outcome.new_item_count, 0);
      setError(null);
      setMessage(
        outcomes.some((outcome) => outcome.status === "error")
          ? "Refresh finished with errors. The source status was updated."
          : `Refresh finished. ${added} new article${added === 1 ? "" : "s"} stored locally.`,
      );
      invalidateRelated();
    },
    onError: (mutationError) => setError(errorMessage(mutationError)),
  });

  const deleteMutation = useMutation({
    mutationFn: (source: IntegrationSource) => desktop.integrationDelete(source.id),
    onSuccess: (_, source) => {
      setError(null);
      setMessage(integrationRemoveMessage(source));
      invalidateRelated();
    },
    onError: (mutationError) => setError(errorMessage(mutationError)),
  });

  const requestDelete = (source: IntegrationSource) => {
    if (!window.confirm(integrationDeleteMessage(source))) return;
    deleteMutation.mutate(source);
  };

  const requestPreview = () => {
    if (!kind || validationError || !input.trim()) return;
    setPreview(null);
    previewMutation.mutate({ platform: kind as IntegrationKind, value: input });
  };

  return (
    <div>
      <p className="bergag-eyebrow">Integrations</p>
      <h2 className="bergag-title bergag-title--small">Public feed connections</h2>
      <p className="bergag-subtitle">
        Add public, feed-backed sources for platforms you use. Bergag fetches only the feed you
        preview, stores metadata in this profile, and reuses the normal reader, search, labels, and
        refresh pipeline. No account or credential is used.
      </p>

      <div className="bergag-card bergag-card--padded" style={{ marginTop: 18 }}>
        <div className="bergag-switch-row">
          <div className="bergag-switch-row__text">
            <span className="bergag-switch-row__label">Available adapters</span>
            <span className="bergag-switch-row__hint">
              YouTube, Reddit, GitHub releases, Hacker News, arXiv, podcasts, Medium, Substack,
              Mastodon public posts, public-newsletter RSS, and website feeds.
            </span>
          </div>
          <span className="bergag-badge bergag-badge--accent">{available.length}</span>
        </div>
        <div className="bergag-switch-row">
          <div className="bergag-switch-row__text">
            <span className="bergag-switch-row__label">Credentials</span>
            <span className="bergag-switch-row__hint">
              None. Private or authenticated sources are not supported in this build.
            </span>
          </div>
          <span className="bergag-badge">Not used</span>
        </div>
        <div className="bergag-switch-row">
          <div className="bergag-switch-row__text">
            <span className="bergag-switch-row__label">Planned adapters</span>
            <span className="bergag-switch-row__hint">
              Bluesky and true website change monitoring are not available and make no requests.
            </span>
          </div>
          <span className="bergag-badge">{planned.length} planned</span>
        </div>
      </div>

      <h3 className="bergag-section-title">Add a source</h3>
      <div className="bergag-card bergag-card--padded">
        {platformsQuery.isLoading ? <div className="bergag-loading">Loading platforms…</div> : null}
        {platformsQuery.isError ? (
          <div className="bergag-error" role="alert">
            {errorMessage(platformsQuery.error)}
          </div>
        ) : null}
        <div className="bergag-field">
          <label className="bergag-field__label" htmlFor="integration-kind">
            Platform
          </label>
          <select
            id="integration-kind"
            className="bergag-select"
            value={kind}
            onChange={(event) => {
              setSelectedKind(event.target.value as IntegrationKind | "");
              setPreview(null);
              setError(null);
            }}
          >
            <option value="">Choose a platform</option>
            {available.map((platform) => (
              <option key={platform.kind} value={platform.kind}>
                {platform.label}
              </option>
            ))}
            {planned.map((platform) => (
              <option
                key={platform.kind}
                value={platform.kind}
                disabled
                title={platform.unavailable_reason ?? "Planned"}
              >
                {platform.label} (planned)
              </option>
            ))}
          </select>
          <div className="bergag-field__hint">
            {selected?.description ?? "Choose a public, feed-backed platform."}
          </div>
        </div>

        <div className="bergag-field">
          <label className="bergag-field__label" htmlFor="integration-input">
            {selected?.input_label ?? "Source"}
          </label>
          <input
            id="integration-input"
            className="bergag-input"
            value={input}
            placeholder={selected?.input_hint ?? "Choose a platform first."}
            onChange={(event) => {
              setInput(event.target.value);
              setPreview(null);
              setError(null);
            }}
          />
          <div className="bergag-field__hint">{selected?.input_hint ?? ""}</div>
        </div>

        <div className="bergag-field">
          <label className="bergag-field__label" htmlFor="integration-category">
            Category (optional)
          </label>
          <input
            id="integration-category"
            className="bergag-input"
            value={category}
            placeholder="For example, News"
            onChange={(event) => setCategory(event.target.value)}
          />
        </div>

        <div className="bergag-field">
          <label className="bergag-field__label" htmlFor="integration-trust">
            Trust level
          </label>
          <select
            id="integration-trust"
            className="bergag-select"
            value={trustLevel}
            onChange={(event) => setTrustLevel(event.target.value as TrustLevel)}
          >
            {TRUST_LEVELS.map((level) => (
              <option key={level} value={level}>
                {level.charAt(0).toUpperCase() + level.slice(1)}
              </option>
            ))}
          </select>
        </div>

        {validationError ? <div className="bergag-field__hint">{validationError}</div> : null}
        {error ? (
          <div className="bergag-error" role="alert">
            {error}
          </div>
        ) : null}
        {message ? <div className="bergag-note">{message}</div> : null}

        <div className="bergag-inline-actions">
          <Button
            size="sm"
            disabled={Boolean(validationError) || previewMutation.isPending || !input.trim()}
            onClick={requestPreview}
          >
            {previewMutation.isPending ? "Previewing…" : "Preview feed"}
          </Button>
          <Button
            size="sm"
            variant="primary"
            disabled={!previewMatches || addMutation.isPending}
            onClick={() => addMutation.mutate()}
          >
            {addMutation.isPending ? "Adding…" : "Add & subscribe"}
          </Button>
        </div>
      </div>

      {preview ? (
        <div className="bergag-card bergag-card--padded" style={{ marginTop: 16 }}>
          <div className="bergag-stat__label">Preview</div>
          <div className="bergag-title bergag-title--small">{preview.result.preview.title}</div>
          <div className="bergag-stat__detail">
            {preview.result.preview.feed_type.toUpperCase()}
          </div>
          <div className="bergag-field__hint">{preview.result.preview.item_count} feed items</div>
          <div className="bergag-field__hint">{preview.result.resolved_feed_url}</div>
          {integrationPreviewNotices(preview.result).map((notice) => (
            <div className="bergag-note" key={notice}>
              {notice}
            </div>
          ))}
        </div>
      ) : null}

      <h3 className="bergag-section-title">Connected sources</h3>
      {sourcesQuery.isLoading ? <div className="bergag-loading">Loading sources…</div> : null}
      {sourcesQuery.isError ? (
        <div className="bergag-error" role="alert">
          {errorMessage(sourcesQuery.error)}
        </div>
      ) : null}
      {sources.length === 0 && !sourcesQuery.isLoading ? (
        <div className="bergag-note">No integrations are connected in this profile.</div>
      ) : null}
      <div className="bergag-ai-provider-list">
        {sources.map((source) => (
          <div className="bergag-ai-provider" key={source.id}>
            <div className="bergag-ai-provider__head">
              <div>
                <div className="bergag-stat__label">{source.display_name}</div>
                <div className="bergag-stat__detail">
                  {integrationKindLabel(platforms, source.kind)}
                  {source.category ? ` · ${source.category}` : ""}
                </div>
                <div className="bergag-field__hint">{source.feed_url}</div>
                {source.last_error ? (
                  <div className="bergag-field__hint">Last error: {source.last_error}</div>
                ) : null}
              </div>
              <div className="bergag-inline-actions">
                <span className="bergag-badge">{source.status}</span>
                <span className="bergag-badge bergag-badge--accent">{source.trust_level}</span>
              </div>
            </div>
            <div className="bergag-inline-actions">
              <select
                className="bergag-select"
                aria-label={`Trust for ${source.display_name}`}
                value={source.trust_level}
                disabled={trustMutation.isPending}
                onChange={(event) =>
                  trustMutation.mutate({
                    sourceId: source.id,
                    level: event.target.value as TrustLevel,
                  })
                }
              >
                {TRUST_LEVELS.map((level) => (
                  <option key={level} value={level}>
                    {level.charAt(0).toUpperCase() + level.slice(1)}
                  </option>
                ))}
              </select>
              <Button
                size="sm"
                disabled={!source.feed_id || refreshMutation.isPending}
                onClick={() => refreshMutation.mutate(source.id)}
              >
                Refresh now
              </Button>
              <Button
                size="sm"
                variant="danger"
                disabled={deleteMutation.isPending}
                onClick={() => requestDelete(source)}
              >
                Remove
              </Button>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
