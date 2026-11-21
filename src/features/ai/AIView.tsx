import { useProfile } from "../../app/ProfileProvider";
import { errorMessage } from "../../application/articleService";
import {
  AI_FEATURE_LABELS,
  aiReadiness,
  canRunAiFeature,
  selectedAiProvider,
  type AiFeatureId,
} from "../../application/aiService";
import { queryKeys } from "../../application/queryKeys";
import { Button } from "../../components/ui/Button";
import type { AiCompletion, ArticleListItem } from "../../domain/models";
import { normalizeAi } from "../../domain/settings";
import { desktop } from "../../infrastructure/tauri/commands";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";

interface AIViewProps {
  onOpenSettings: () => void;
}

interface RunRequest {
  feature: AiFeatureId;
  itemId: string;
  question?: string;
}

export function AIView({ onOpenSettings }: AIViewProps) {
  const { activeProfile } = useProfile();
  const profileId = activeProfile?.id ?? "";
  const queryClient = useQueryClient();
  const [selectedItemId, setSelectedItemId] = useState<string | null>(null);
  const [question, setQuestion] = useState("");
  const [completion, setCompletion] = useState<AiCompletion | null>(null);
  const [error, setError] = useState<string | null>(null);

  const settingsQuery = useQuery({
    queryKey: queryKeys.aiSettings(profileId),
    queryFn: desktop.aiSettingsGet,
    enabled: Boolean(profileId),
  });
  const providersQuery = useQuery({
    queryKey: queryKeys.aiProviders(profileId),
    queryFn: desktop.aiProvidersList,
    enabled: Boolean(profileId),
  });
  const articlesQuery = useQuery({
    queryKey: queryKeys.aiArticles(profileId),
    queryFn: () => desktop.listArticles("all", null, 200),
    enabled: Boolean(profileId),
  });
  const usageQuery = useQuery({
    queryKey: queryKeys.aiUsageStats(profileId, 30),
    queryFn: () => desktop.aiUsageStats(30),
    enabled: Boolean(profileId),
  });

  const settings = normalizeAi(settingsQuery.data);
  const providers = providersQuery.data ?? [];
  const articles = articlesQuery.data ?? [];
  const provider = selectedAiProvider(settings, providers);
  const itemId = selectedItemId ?? articles[0]?.id ?? null;
  const article = articles.find((entry) => entry.id === itemId) ?? null;

  const runMutation = useMutation({
    mutationFn: (request: RunRequest) => {
      if (request.feature === "summarize") return desktop.aiSummarize(request.itemId);
      if (request.feature === "suggestLabels") return desktop.aiSuggestLabels(request.itemId);
      return desktop.aiAsk(request.itemId, request.question ?? "");
    },
    onSuccess: (result) => {
      setCompletion(result);
      setError(null);
      void queryClient.invalidateQueries({ queryKey: queryKeys.aiUsageStats(profileId, 30) });
    },
    onError: (mutationError) => setError(errorMessage(mutationError)),
  });

  const requestRun = (feature: AiFeatureId) => {
    const issues = aiReadiness(settings, providers, feature);
    if (issues.length > 0) {
      setError(issues.join(" "));
      return;
    }
    if (!itemId) {
      setError("Add or select an article before using AI features.");
      return;
    }
    if (feature === "ask" && question.trim().length === 0) {
      setError("Enter a question about the selected article.");
      return;
    }
    if (settings.privacy.requireConfirmation) {
      const destination = provider ? `${provider.name} (${provider.endpoint})` : "the AI provider";
      const detail =
        feature === "ask" ? `question: ${question.trim()}` : `article: ${article?.title ?? itemId}`;
      const confirmed = window.confirm(
        `Send the selected local article context to ${destination}?\n\n${detail}\n\nNo content is sent unless you confirm.`,
      );
      if (!confirmed) return;
    }
    setCompletion(null);
    setError(null);
    runMutation.mutate({
      feature,
      itemId,
      question: feature === "ask" ? question.trim() : undefined,
    });
  };

  const loading = settingsQuery.isLoading || providersQuery.isLoading || articlesQuery.isLoading;
  const queryError =
    settingsQuery.error ?? providersQuery.error ?? articlesQuery.error ?? usageQuery.error;

  return (
    <div className="bergag-full-view">
      <div className="bergag-full-view__header">
        <div>
          <h2 className="bergag-title">AI</h2>
          <p className="bergag-subtitle">
            Explicit, opt-in help for your local library. No background processing, no automatic
            uploads, and no credentials exposed to this interface.
          </p>
        </div>
        <Button variant="primary" onClick={onOpenSettings}>
          Configure AI
        </Button>
      </div>
      <div className="bergag-full-view__body">
        <div className="bergag-ai-panel">
          {loading ? <div className="bergag-loading">Loading AI state…</div> : null}
          {queryError ? (
            <div className="bergag-error" role="alert">
              {errorMessage(queryError)}
            </div>
          ) : null}

          <div className="bergag-card bergag-card--padded">
            <h3 className="bergag-section-title">Current provider</h3>
            {provider ? (
              <div className="bergag-ai-provider__head">
                <div>
                  <div className="bergag-stat__label">{provider.name}</div>
                  <div className="bergag-stat__detail">
                    {provider.model} · {provider.endpoint}
                  </div>
                </div>
                <span className="bergag-badge">
                  {settings.enabled && provider.enabled ? "Ready" : "Not ready"}
                </span>
              </div>
            ) : (
              <p className="bergag-subtitle">
                No provider is selected. Add and select one in AI settings before running a feature.
              </p>
            )}
            {!settings.enabled ? (
              <div className="bergag-note">AI is currently disabled in settings.</div>
            ) : null}
          </div>

          <div className="bergag-card bergag-card--padded">
            <h3 className="bergag-section-title">Choose an article</h3>
            {articles.length === 0 ? (
              <p className="bergag-subtitle">Your library has no articles yet.</p>
            ) : (
              <div className="bergag-field">
                <label className="bergag-field__label" htmlFor="ai-article">
                  Article
                </label>
                <select
                  id="ai-article"
                  className="bergag-select"
                  value={itemId ?? ""}
                  onChange={(event) => setSelectedItemId(event.target.value)}
                >
                  {articles.map((entry: ArticleListItem) => (
                    <option value={entry.id} key={entry.id}>
                      {entry.title}
                    </option>
                  ))}
                </select>
              </div>
            )}

            <div className="bergag-field">
              <label className="bergag-field__label" htmlFor="ai-question">
                Question for “Ask about article”
              </label>
              <textarea
                id="ai-question"
                className="bergag-input"
                rows={3}
                maxLength={1000}
                value={question}
                onChange={(event) => setQuestion(event.target.value)}
                placeholder="Ask only about the selected article’s local text."
              />
              <span className="bergag-field__hint">
                The question is sent only when you click Ask and confirm the destination.
              </span>
            </div>

            <div className="bergag-inline-actions">
              {(["summarize", "suggestLabels", "ask"] as AiFeatureId[]).map((feature) => {
                const issues = aiReadiness(settings, providers, feature);
                const blocked = issues.length > 0 || !itemId || runMutation.isPending;
                return (
                  <Button
                    key={feature}
                    variant={feature === "summarize" ? "primary" : "secondary"}
                    disabled={blocked}
                    title={issues.length > 0 ? issues.join(" ") : AI_FEATURE_LABELS[feature]}
                    onClick={() => requestRun(feature)}
                  >
                    {runMutation.isPending ? "Working…" : AI_FEATURE_LABELS[feature]}
                  </Button>
                );
              })}
            </div>
            {itemId && !canRunAiFeature(settings, providers, "summarize") ? (
              <div className="bergag-note" style={{ marginTop: 12 }}>
                Some features are blocked by privacy or provider settings. Configure AI to enable
                them; nothing is sent while a feature is blocked.
              </div>
            ) : null}
          </div>

          {completion ? (
            <div className="bergag-card bergag-card--padded">
              <div className="bergag-ai-result__meta">
                <span className="bergag-badge bergag-badge--accent">{completion.feature}</span>
                <span className="bergag-stat__detail">
                  {completion.provider_name} · {completion.model} · {completion.latency_ms} ms
                </span>
              </div>
              <pre className="bergag-ai-result">{completion.text}</pre>
              <div className="bergag-field__hint">
                Token counts reported by the provider: {completion.prompt_tokens} prompt /{" "}
                {completion.completion_tokens} completion.
              </div>
            </div>
          ) : null}

          {error ? (
            <p className="bergag-error" role="alert">
              {error}
            </p>
          ) : null}

          <div className="bergag-card bergag-card--padded">
            <h3 className="bergag-section-title">Local usage this month</h3>
            {usageQuery.isLoading ? <div className="bergag-loading">Loading usage…</div> : null}
            {usageQuery.data ? (
              <>
                <div className="bergag-card-grid">
                  <div className="bergag-card bergag-card--padded">
                    <div className="bergag-stat__label">Requests</div>
                    <div className="bergag-stat__value">{usageQuery.data.total_requests}</div>
                  </div>
                  <div className="bergag-card bergag-card--padded">
                    <div className="bergag-stat__label">Successful</div>
                    <div className="bergag-stat__value">{usageQuery.data.successful_requests}</div>
                  </div>
                  <div className="bergag-card bergag-card--padded">
                    <div className="bergag-stat__label">Failed</div>
                    <div className="bergag-stat__value">{usageQuery.data.failed_requests}</div>
                  </div>
                </div>
                <div className="bergag-note">
                  These counts are read from SQLite usage events created by your own AI actions.
                </div>
              </>
            ) : null}
          </div>
        </div>
      </div>
    </div>
  );
}
