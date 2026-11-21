import { useProfile } from "../../app/ProfileProvider";
import { errorMessage } from "../../application/articleService";
import { AI_PROVIDER_KINDS, AI_PROVIDER_PRESETS } from "../../application/aiService";
import { queryKeys } from "../../application/queryKeys";
import { formatRelativeTime } from "../../application/feedService";
import { Button } from "../../components/ui/Button";
import { SwitchField } from "../../components/ui/SwitchField";
import type { AiProvider, AiProviderKind, AiSettings } from "../../domain/models";
import { normalizeAi } from "../../domain/settings";
import { desktop } from "../../infrastructure/tauri/commands";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";

interface ProviderForm {
  id: string | null;
  name: string;
  kind: AiProviderKind;
  endpoint: string;
  model: string;
  enabled: boolean;
  apiKey: string;
}

function emptyForm(kind: AiProviderKind = "openai"): ProviderForm {
  const preset = AI_PROVIDER_PRESETS[kind];
  return {
    id: null,
    name: preset.label,
    kind,
    endpoint: preset.endpoint,
    model: preset.model,
    enabled: true,
    apiKey: "",
  };
}

function formFromProvider(provider: AiProvider): ProviderForm {
  return {
    id: provider.id,
    name: provider.name,
    kind: provider.kind,
    endpoint: provider.endpoint,
    model: provider.model,
    enabled: provider.enabled,
    apiKey: "",
  };
}

export function AISection() {
  const { activeProfile } = useProfile();
  const profileId = activeProfile?.id ?? "";
  const queryClient = useQueryClient();
  const [form, setForm] = useState<ProviderForm>(() => emptyForm());
  const [error, setError] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [testResult, setTestResult] = useState<string | null>(null);

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
  const usageQuery = useQuery({
    queryKey: queryKeys.aiUsageStats(profileId, 30),
    queryFn: () => desktop.aiUsageStats(30),
    enabled: Boolean(profileId),
  });

  const settings = normalizeAi(settingsQuery.data);
  const providers = providersQuery.data ?? [];
  const preset = AI_PROVIDER_PRESETS[form.kind];

  const refresh = () => {
    void queryClient.invalidateQueries({ queryKey: queryKeys.aiSettings(profileId) });
    void queryClient.invalidateQueries({ queryKey: queryKeys.aiProviders(profileId) });
    void queryClient.invalidateQueries({ queryKey: queryKeys.aiUsageStats(profileId, 30) });
  };

  const settingsMutation = useMutation({
    mutationFn: (next: AiSettings) => desktop.aiSettingsUpdate(next),
    onSuccess: (next) => {
      queryClient.setQueryData(queryKeys.aiSettings(profileId), next);
      setError(null);
      setMessage("AI settings saved.");
      refresh();
    },
    onError: (mutationError) => setError(errorMessage(mutationError)),
  });

  const providerMutation = useMutation({
    mutationFn: async () => {
      const saved = await desktop.aiProviderSave(
        form.id,
        form.name,
        form.kind,
        form.endpoint,
        form.model,
        form.enabled,
      );
      if (form.apiKey.trim().length > 0 && saved.requires_key) {
        await desktop.aiSetApiKey(saved.id, form.apiKey.trim());
      }
      return saved;
    },
    onSuccess: () => {
      setForm(emptyForm(form.kind));
      setError(null);
      setMessage("Provider saved. No key value is kept in the interface.");
      refresh();
    },
    onError: (mutationError) => setError(errorMessage(mutationError)),
  });

  const deleteMutation = useMutation({
    mutationFn: (providerId: string) => desktop.aiProviderDelete(providerId),
    onSuccess: () => {
      setForm((current) => (current.id ? emptyForm(current.kind) : current));
      setError(null);
      setMessage("Provider removed, including its stored credential.");
      refresh();
    },
    onError: (mutationError) => setError(errorMessage(mutationError)),
  });

  const keyMutation = useMutation({
    mutationFn: ({ providerId, apiKey }: { providerId: string; apiKey: string | null }) =>
      apiKey === null ? desktop.aiClearApiKey(providerId) : desktop.aiSetApiKey(providerId, apiKey),
    onSuccess: () => {
      setError(null);
      setMessage("Credential updated in the operating system store.");
      refresh();
    },
    onError: (mutationError) => setError(errorMessage(mutationError)),
  });

  const testMutation = useMutation({
    mutationFn: (providerId: string) => desktop.aiProviderTest(providerId),
    onSuccess: (result) => {
      setTestResult(
        `${result.ok ? "Connected" : "Failed"} in ${result.latency_ms} ms: ${result.message}`,
      );
      refresh();
    },
    onError: (mutationError) => setError(errorMessage(mutationError)),
  });

  const clearUsageMutation = useMutation({
    mutationFn: desktop.aiUsageClear,
    onSuccess: (removed) => {
      setMessage(`Removed ${removed} local AI usage event${removed === 1 ? "" : "s"}.`);
      setError(null);
      refresh();
    },
    onError: (mutationError) => setError(errorMessage(mutationError)),
  });

  const patchSettings = (patch: Partial<AiSettings>) => {
    settingsMutation.mutate({
      ...settings,
      ...patch,
      features: { ...settings.features, ...patch.features },
      privacy: { ...settings.privacy, ...patch.privacy },
    });
  };

  const selectKind = (kind: AiProviderKind) => {
    const next = emptyForm(kind);
    setForm((current) => ({
      ...next,
      id: current.id,
      name: current.id ? current.name : next.name,
    }));
  };

  const requestDelete = (provider: AiProvider) => {
    if (!window.confirm(`Remove provider “${provider.name}” and its stored credential?`)) return;
    deleteMutation.mutate(provider.id);
  };

  const requestClearUsage = () => {
    if (!window.confirm("Delete all AI usage events recorded for this profile?")) return;
    clearUsageMutation.mutate();
  };

  return (
    <div>
      <p className="bergag-eyebrow">AI</p>
      <h2 className="bergag-title bergag-title--small">Optional, explicit AI</h2>
      <p className="bergag-subtitle">
        AI is off until you turn it on. Bergag never runs background AI, never sends article content
        automatically, and stores API keys only in the Windows credential store. Every feature below
        is a direct user action.
      </p>

      <div className="bergag-card bergag-card--padded" style={{ marginTop: 18 }}>
        <SwitchField
          label="Enable AI features"
          hint="Master switch for this profile. Disabling it blocks every AI action."
          checked={settings.enabled}
          disabled={settingsMutation.isPending}
          onCheckedChange={(checked) => patchSettings({ enabled: checked })}
        />
        <SwitchField
          label="Summarize articles"
          hint="Sends the selected article text when you click Summarize."
          checked={settings.features.summarize}
          disabled={settingsMutation.isPending}
          onCheckedChange={(checked) =>
            patchSettings({ features: { ...settings.features, summarize: checked } })
          }
        />
        <SwitchField
          label="Suggest labels"
          hint="Sends article text and your existing label names when you click Suggest labels."
          checked={settings.features.suggestLabels}
          disabled={settingsMutation.isPending}
          onCheckedChange={(checked) =>
            patchSettings({ features: { ...settings.features, suggestLabels: checked } })
          }
        />
        <SwitchField
          label="Article questions"
          hint="Sends the selected article and your question when you click Ask."
          checked={settings.features.ask}
          disabled={settingsMutation.isPending}
          onCheckedChange={(checked) =>
            patchSettings({ features: { ...settings.features, ask: checked } })
          }
        />
      </div>

      <h3 className="bergag-section-title">Privacy boundaries</h3>
      <div className="bergag-card bergag-card--padded">
        <SwitchField
          label="Allow article text"
          hint="Required before any article feature can send local text to a provider."
          checked={settings.privacy.allowArticleText}
          disabled={settingsMutation.isPending}
          onCheckedChange={(checked) =>
            patchSettings({ privacy: { ...settings.privacy, allowArticleText: checked } })
          }
        />
        <SwitchField
          label="Allow saved highlights"
          hint="Includes your highlights in the prompt for the selected article."
          checked={settings.privacy.allowHighlights}
          disabled={settingsMutation.isPending}
          onCheckedChange={(checked) =>
            patchSettings({ privacy: { ...settings.privacy, allowHighlights: checked } })
          }
        />
        <SwitchField
          label="Allow private notes"
          hint="Includes your notes in the prompt for the selected article."
          checked={settings.privacy.allowNotes}
          disabled={settingsMutation.isPending}
          onCheckedChange={(checked) =>
            patchSettings({ privacy: { ...settings.privacy, allowNotes: checked } })
          }
        />
        <SwitchField
          label="Allow label names"
          hint="Lets label suggestions choose only from your existing tag and topic names."
          checked={settings.privacy.allowLabels}
          disabled={settingsMutation.isPending}
          onCheckedChange={(checked) =>
            patchSettings({ privacy: { ...settings.privacy, allowLabels: checked } })
          }
        />
        <SwitchField
          label="Allow local providers"
          hint="Permits a narrowly scoped Ollama loopback endpoint. No other local network target is allowed."
          checked={settings.privacy.localProvidersEnabled}
          disabled={settingsMutation.isPending}
          onCheckedChange={(checked) =>
            patchSettings({ privacy: { ...settings.privacy, localProvidersEnabled: checked } })
          }
        />
        <SwitchField
          label="Ask before sending"
          hint="Shows a confirmation containing the destination before every AI action."
          checked={settings.privacy.requireConfirmation}
          disabled={settingsMutation.isPending}
          onCheckedChange={(checked) =>
            patchSettings({ privacy: { ...settings.privacy, requireConfirmation: checked } })
          }
        />
      </div>

      <h3 className="bergag-section-title">Providers</h3>
      <p className="bergag-subtitle">
        Remote endpoints must be public HTTPS. Ollama is the only local exception and requires the
        opt-in above plus a literal loopback address.
      </p>

      {providersQuery.isLoading ? <div className="bergag-loading">Loading providers…</div> : null}
      {providersQuery.isError ? (
        <div className="bergag-error" role="alert">
          {errorMessage(providersQuery.error)}
        </div>
      ) : null}
      {providers.length === 0 && !providersQuery.isLoading ? (
        <div className="bergag-note">No AI providers are configured for this profile.</div>
      ) : null}

      <div className="bergag-ai-provider-list">
        {providers.map((provider) => (
          <div className="bergag-ai-provider" key={provider.id}>
            <div className="bergag-ai-provider__head">
              <div>
                <div className="bergag-stat__label">{provider.name}</div>
                <div className="bergag-stat__detail">
                  {AI_PROVIDER_PRESETS[provider.kind].label} · {provider.model}
                </div>
                <div className="bergag-field__hint">{provider.endpoint}</div>
              </div>
              <div className="bergag-inline-actions">
                <span className="bergag-badge">{provider.enabled ? "Enabled" : "Disabled"}</span>
                <span className="bergag-badge bergag-badge--accent">
                  {provider.has_key
                    ? "Key stored"
                    : provider.requires_key
                      ? "Key missing"
                      : "No key needed"}
                </span>
              </div>
            </div>
            <div className="bergag-inline-actions">
              <Button
                size="sm"
                disabled={settingsMutation.isPending}
                onClick={() =>
                  patchSettings({
                    providerId: settings.providerId === provider.id ? null : provider.id,
                  })
                }
              >
                {settings.providerId === provider.id ? "Deselect" : "Select"}
              </Button>
              <Button size="sm" onClick={() => setForm(formFromProvider(provider))}>
                Edit
              </Button>
              <Button
                size="sm"
                disabled={testMutation.isPending}
                onClick={() => testMutation.mutate(provider.id)}
              >
                {testMutation.isPending ? "Testing…" : "Test connection"}
              </Button>
              {provider.requires_key ? (
                <Button
                  size="sm"
                  disabled={keyMutation.isPending}
                  onClick={() => {
                    const key = window.prompt(
                      `Enter the API key for ${provider.name}. It is stored only in Windows Credential Manager.`,
                    );
                    if (key && key.trim()) {
                      keyMutation.mutate({ providerId: provider.id, apiKey: key.trim() });
                    }
                  }}
                >
                  {provider.has_key ? "Replace key" : "Store key"}
                </Button>
              ) : null}
              {provider.has_key ? (
                <Button
                  size="sm"
                  variant="ghost"
                  disabled={keyMutation.isPending}
                  onClick={() => {
                    if (!window.confirm(`Remove the stored key for ${provider.name}?`)) return;
                    keyMutation.mutate({ providerId: provider.id, apiKey: null });
                  }}
                >
                  Remove key
                </Button>
              ) : null}
              <Button size="sm" variant="danger" onClick={() => requestDelete(provider)}>
                Delete
              </Button>
            </div>
          </div>
        ))}
      </div>

      <h3 className="bergag-section-title">{form.id ? "Edit provider" : "Add provider"}</h3>
      <div className="bergag-card bergag-card--padded">
        <div className="bergag-field">
          <label className="bergag-field__label" htmlFor="ai-provider-kind">
            Provider type
          </label>
          <select
            id="ai-provider-kind"
            className="bergag-select"
            value={form.kind}
            onChange={(event) => selectKind(event.target.value as AiProviderKind)}
          >
            {AI_PROVIDER_KINDS.map((kind) => (
              <option value={kind} key={kind}>
                {AI_PROVIDER_PRESETS[kind].label}
              </option>
            ))}
          </select>
        </div>
        <div className="bergag-field">
          <label className="bergag-field__label" htmlFor="ai-provider-name">
            Display name
          </label>
          <input
            id="ai-provider-name"
            className="bergag-input"
            value={form.name}
            onChange={(event) => setForm({ ...form, name: event.target.value })}
            maxLength={64}
          />
        </div>
        <div className="bergag-field">
          <label className="bergag-field__label" htmlFor="ai-provider-endpoint">
            Endpoint
          </label>
          <input
            id="ai-provider-endpoint"
            className="bergag-input"
            value={form.endpoint}
            onChange={(event) => setForm({ ...form, endpoint: event.target.value })}
            maxLength={512}
          />
          <span className="bergag-field__hint">
            {form.kind === "ollama"
              ? "Only http://127.0.0.1:<port> or http://[::1]:<port> is accepted."
              : "Public HTTPS URLs only; redirects to private addresses are blocked."}
          </span>
        </div>
        <div className="bergag-field">
          <label className="bergag-field__label" htmlFor="ai-provider-model">
            Model
          </label>
          <input
            id="ai-provider-model"
            className="bergag-input"
            value={form.model}
            onChange={(event) => setForm({ ...form, model: event.target.value })}
            maxLength={128}
          />
        </div>
        {preset.requiresKey ? (
          <div className="bergag-field">
            <label className="bergag-field__label" htmlFor="ai-provider-key">
              API key {form.id ? "(leave blank to keep the stored key)" : ""}
            </label>
            <input
              id="ai-provider-key"
              className="bergag-input"
              type="password"
              autoComplete="off"
              value={form.apiKey}
              onChange={(event) => setForm({ ...form, apiKey: event.target.value })}
              placeholder="Stored in Windows Credential Manager"
            />
          </div>
        ) : null}
        <SwitchField
          label="Provider enabled"
          hint="Disabled providers cannot be selected for AI actions."
          checked={form.enabled}
          onCheckedChange={(checked) => setForm({ ...form, enabled: checked })}
        />
        <div className="bergag-inline-actions" style={{ marginTop: 12 }}>
          <Button
            variant="primary"
            disabled={providerMutation.isPending || form.name.trim().length === 0}
            onClick={() => providerMutation.mutate()}
          >
            {providerMutation.isPending ? "Saving…" : form.id ? "Save provider" : "Add provider"}
          </Button>
          {form.id ? (
            <Button variant="ghost" onClick={() => setForm(emptyForm(form.kind))}>
              Cancel edit
            </Button>
          ) : null}
        </div>
      </div>

      <h3 className="bergag-section-title">AI privacy dashboard</h3>
      {usageQuery.isLoading ? <div className="bergag-loading">Loading AI usage…</div> : null}
      {usageQuery.isError ? (
        <div className="bergag-error" role="alert">
          {errorMessage(usageQuery.error)}
        </div>
      ) : null}
      {usageQuery.data ? (
        <div className="bergag-ai-usage">
          <div className="bergag-card-grid">
            <div className="bergag-card bergag-card--padded">
              <div className="bergag-stat__label">Requests (30 days)</div>
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
            <div className="bergag-card bergag-card--padded">
              <div className="bergag-stat__label">Last request</div>
              <div className="bergag-stat__value">
                {formatRelativeTime(usageQuery.data.last_used_at)}
              </div>
            </div>
          </div>
          <div className="bergag-note">
            Counts come from local usage events only. Token totals are shown only when a provider
            reports them: {usageQuery.data.prompt_tokens} prompt /{" "}
            {usageQuery.data.completion_tokens} completion tokens. Character totals:{" "}
            {usageQuery.data.total_input_chars} sent / {usageQuery.data.total_output_chars}{" "}
            received.
          </div>
          <div className="bergag-inline-actions" style={{ marginTop: 8 }}>
            <Button
              variant="danger"
              size="sm"
              disabled={clearUsageMutation.isPending}
              onClick={requestClearUsage}
            >
              {clearUsageMutation.isPending ? "Clearing…" : "Clear local AI history"}
            </Button>
          </div>
        </div>
      ) : null}

      {testResult ? (
        <div className="bergag-note" style={{ marginTop: 12 }}>
          {testResult}
        </div>
      ) : null}
      {message ? (
        <div className="bergag-note" style={{ marginTop: 12 }}>
          {message}
        </div>
      ) : null}
      {error ? (
        <p className="bergag-error" role="alert" style={{ marginTop: 12 }}>
          {error}
        </p>
      ) : null}
    </div>
  );
}
