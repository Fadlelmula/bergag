import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { History, LoaderCircle, Pencil, Play, Plus, Trash2, WandSparkles } from "lucide-react";
import { useState } from "react";
import { ruleSummary } from "../../application/discoveryService";
import { queryKeys } from "../../application/queryKeys";
import { Button } from "../../components/ui/Button";
import { IconButton } from "../../components/ui/IconButton";
import type { Rule, RuleRun } from "../../domain/models";
import { desktop } from "../../infrastructure/tauri/commands";
import { RuleBuilderDialog } from "./RuleBuilderDialog";

interface RulesPanelProps {
  profileId: string;
  onOpenArticle: (id: string) => void;
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function formatTime(value: string | null): string {
  if (!value) return "Never";
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString();
}

function runSummary(run: RuleRun): string {
  return `${run.matched_count} matched · ${run.changed_count} changed`;
}

export function RulesPanel({ profileId, onOpenArticle }: RulesPanelProps) {
  const queryClient = useQueryClient();
  const rules = useQuery({
    queryKey: queryKeys.rules(profileId),
    queryFn: desktop.listRules,
  });
  const feeds = useQuery({
    queryKey: queryKeys.feeds(profileId),
    queryFn: desktop.listFeeds,
  });
  const labels = useQuery({
    queryKey: queryKeys.labels(profileId),
    queryFn: () => desktop.listLabels(null),
  });
  const [builderOpen, setBuilderOpen] = useState(false);
  const [editingRule, setEditingRule] = useState<Rule | null>(null);
  const [selectedRuleId, setSelectedRuleId] = useState<string | null>(null);
  const [selectedSmartFeedId, setSelectedSmartFeedId] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);

  const history = useQuery({
    queryKey: queryKeys.ruleHistory(profileId, selectedRuleId ?? ""),
    queryFn: () => desktop.listRuleHistory(selectedRuleId ?? ""),
    enabled: Boolean(selectedRuleId),
  });
  const smartFeed = useQuery({
    queryKey: queryKeys.smartFeed(profileId, selectedSmartFeedId ?? ""),
    queryFn: () => desktop.smartFeedItems(selectedSmartFeedId ?? ""),
    enabled: Boolean(selectedSmartFeedId),
  });

  const invalidate = async () => {
    await Promise.all([
      queryClient.invalidateQueries({ queryKey: queryKeys.rules(profileId) }),
      queryClient.invalidateQueries({ queryKey: ["articles"] }),
      queryClient.invalidateQueries({ queryKey: queryKeys.labels(profileId) }),
    ]);
    if (selectedRuleId) {
      await queryClient.invalidateQueries({
        queryKey: queryKeys.ruleHistory(profileId, selectedRuleId),
      });
    }
    if (selectedSmartFeedId) {
      await queryClient.invalidateQueries({
        queryKey: queryKeys.smartFeed(profileId, selectedSmartFeedId),
      });
    }
  };

  const runRule = useMutation({
    mutationFn: (id: string) => desktop.runRule(id),
    onSuccess: async (run) => {
      setNotice(`Rule run: ${runSummary(run)}.`);
      setActionError(null);
      await invalidate();
    },
    onError: (error) => setActionError(errorMessage(error)),
  });

  const runEnabled = useMutation({
    mutationFn: () => desktop.runEnabledRules(),
    onSuccess: async (runs) => {
      setNotice(
        runs.length === 0
          ? "No enabled rules to run."
          : `Ran ${runs.length} enabled rule${runs.length === 1 ? "" : "s"}.`,
      );
      setActionError(null);
      await invalidate();
    },
    onError: (error) => setActionError(errorMessage(error)),
  });

  const remove = useMutation({
    mutationFn: (id: string) => desktop.deleteRule(id),
    onSuccess: async () => {
      setNotice("Rule deleted.");
      await invalidate();
    },
    onError: (error) => setActionError(errorMessage(error)),
  });

  const smartFeeds = (rules.data ?? []).filter((rule) => rule.is_smart_feed);

  return (
    <div className="bergag-rules-panel">
      <div className="bergag-rules-toolbar">
        <div className="bergag-inline-actions">
          <Button
            variant="primary"
            onClick={() => {
              setEditingRule(null);
              setBuilderOpen(true);
            }}
            icon={<Plus size={14} />}
          >
            New rule
          </Button>
          <Button
            variant="secondary"
            onClick={() => runEnabled.mutate()}
            disabled={runEnabled.isPending}
            icon={runEnabled.isPending ? <LoaderCircle size={14} /> : <Play size={14} />}
          >
            Run enabled rules
          </Button>
        </div>
        <span className="bergag-feed-toolbar__count">
          {(rules.data ?? []).length} rule{(rules.data ?? []).length === 1 ? "" : "s"}
        </span>
      </div>

      {notice ? (
        <div className="bergag-feed-notice">
          <span>{notice}</span>
          <Button variant="ghost" size="sm" onClick={() => setNotice(null)}>
            Dismiss
          </Button>
        </div>
      ) : null}
      {actionError ? (
        <div className="bergag-error" role="alert">
          {actionError}
        </div>
      ) : null}
      {rules.error ? (
        <div className="bergag-error" role="alert">
          {errorMessage(rules.error)}
        </div>
      ) : null}

      <div className="bergag-rules-grid">
        <section className="bergag-rules-column" aria-label="Rules">
          <div className="bergag-discovery-column__header">
            <h3 className="bergag-title bergag-title--small">Automation rules</h3>
          </div>
          {rules.isLoading ? <div className="bergag-loading">Loading rules…</div> : null}
          {!rules.isLoading && (rules.data ?? []).length === 0 ? (
            <div className="bergag-empty-state">
              <div className="bergag-empty-state__title">No rules yet</div>
              <p className="bergag-empty-state__text">
                Build a rule from conditions and actions. Rules only run when you choose Run, so
                nothing changes silently.
              </p>
              <Button
                variant="secondary"
                onClick={() => {
                  setEditingRule(null);
                  setBuilderOpen(true);
                }}
                icon={<Plus size={14} />}
              >
                Create a rule
              </Button>
            </div>
          ) : null}
          {(rules.data ?? []).map((rule) => (
            <article className="bergag-rule-card" key={rule.id}>
              <div className="bergag-rule-card__main">
                <div className="bergag-rule-card__title">{rule.name}</div>
                <div className="bergag-rule-card__summary">{ruleSummary(rule)}</div>
                {rule.description ? (
                  <div className="bergag-rule-card__description">{rule.description}</div>
                ) : null}
                <div className="bergag-inline-actions">
                  <span className="bergag-badge">{rule.enabled ? "Enabled" : "Disabled"}</span>
                  {rule.is_smart_feed ? (
                    <span className="bergag-badge">
                      <WandSparkles size={11} aria-hidden="true" /> Smart feed
                    </span>
                  ) : null}
                  <span className="bergag-badge">Last run: {formatTime(rule.last_run_at)}</span>
                </div>
              </div>
              <div className="bergag-rule-card__actions">
                <Button
                  variant="secondary"
                  size="sm"
                  onClick={() => runRule.mutate(rule.id)}
                  disabled={runRule.isPending}
                  icon={<Play size={13} />}
                >
                  Run
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => setSelectedRuleId(rule.id)}
                  icon={<History size={13} />}
                >
                  History
                </Button>
                <IconButton
                  label={`Edit ${rule.name}`}
                  icon={<Pencil size={13} aria-hidden="true" />}
                  onClick={() => {
                    setEditingRule(rule);
                    setBuilderOpen(true);
                  }}
                />
                <IconButton
                  label={`Delete ${rule.name}`}
                  icon={<Trash2 size={13} aria-hidden="true" />}
                  onClick={() => remove.mutate(rule.id)}
                />
              </div>
            </article>
          ))}

          <div className="bergag-rule-history">
            <div className="bergag-discovery-column__header">
              <h3 className="bergag-title bergag-title--small">Rule history</h3>
              {selectedRuleId ? (
                <Button variant="ghost" size="sm" onClick={() => setSelectedRuleId(null)}>
                  Clear
                </Button>
              ) : null}
            </div>
            {!selectedRuleId ? (
              <p className="bergag-empty-state__text">
                Choose History on a rule to inspect local run results.
              </p>
            ) : history.isLoading ? (
              <div className="bergag-loading">Loading run history…</div>
            ) : (history.data ?? []).length === 0 ? (
              <p className="bergag-empty-state__text">This rule has not run yet.</p>
            ) : (
              <div className="bergag-run-list">
                {(history.data ?? []).map((run) => (
                  <div className="bergag-run-row" key={run.id}>
                    <span className={`bergag-run-status bergag-run-status--${run.status}`}>
                      {run.status}
                    </span>
                    <span>{formatTime(run.finished_at)}</span>
                    <span>{runSummary(run)}</span>
                    {run.message ? <span className="bergag-run-message">{run.message}</span> : null}
                  </div>
                ))}
              </div>
            )}
          </div>
        </section>

        <section className="bergag-rules-column" aria-label="Smart feeds">
          <div className="bergag-discovery-column__header">
            <h3 className="bergag-title bergag-title--small">Smart feeds</h3>
          </div>
          {smartFeeds.length === 0 ? (
            <p className="bergag-empty-state__text">
              Mark a rule as a smart feed to collect its matching articles here. Smart feeds are
              views only; they do not modify article state until you run the rule.
            </p>
          ) : (
            <div className="bergag-smart-feed-tabs">
              {smartFeeds.map((rule) => (
                <button
                  type="button"
                  key={rule.id}
                  className={`bergag-smart-feed-tab ${selectedSmartFeedId === rule.id ? "bergag-smart-feed-tab--active" : ""}`}
                  onClick={() => setSelectedSmartFeedId(rule.id)}
                >
                  {rule.name}
                </button>
              ))}
            </div>
          )}
          {selectedSmartFeedId ? (
            smartFeed.isLoading ? (
              <div className="bergag-loading">Loading smart feed…</div>
            ) : (smartFeed.data ?? []).length === 0 ? (
              <p className="bergag-empty-state__text">No articles match this smart feed yet.</p>
            ) : (
              <div className="bergag-smart-feed-list">
                {(smartFeed.data ?? []).map((article) => (
                  <button
                    type="button"
                    className="bergag-smart-feed-item"
                    key={article.id}
                    onClick={() => onOpenArticle(article.id)}
                  >
                    <span className="bergag-smart-feed-item__title">{article.title}</span>
                    <span className="bergag-smart-feed-item__meta">
                      {article.feed_title} · {article.state.is_read ? "Read" : "Unread"}
                    </span>
                  </button>
                ))}
              </div>
            )
          ) : null}
        </section>
      </div>

      {builderOpen ? (
        <RuleBuilderDialog
          key={editingRule?.id ?? "new"}
          open={builderOpen}
          onOpenChange={setBuilderOpen}
          rule={editingRule}
          feeds={feeds.data ?? []}
          labels={labels.data ?? []}
          onSaved={() => {
            void queryClient.invalidateQueries({ queryKey: queryKeys.rules(profileId) });
          }}
        />
      ) : null}
    </div>
  );
}
