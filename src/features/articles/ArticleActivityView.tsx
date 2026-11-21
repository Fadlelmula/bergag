import { errorMessage, formatArticleDate } from "../../application/articleService";
import { queryKeys } from "../../application/queryKeys";
import { Button } from "../../components/ui/Button";
import { IconButton } from "../../components/ui/IconButton";
import { desktop } from "../../infrastructure/tauri/commands";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Trash2 } from "lucide-react";
import { useState } from "react";

interface ArticleActivityViewProps {
  kind: "history" | "highlights";
  profileId: string;
  onSelectArticle: (id: string) => void;
}

export function ArticleActivityView({
  kind,
  profileId,
  onSelectArticle,
}: ArticleActivityViewProps) {
  const queryClient = useQueryClient();
  const [error, setError] = useState<string | null>(null);
  const history = useQuery({
    queryKey: queryKeys.articleHistory(profileId),
    queryFn: () => desktop.listArticleHistory(200),
    enabled: kind === "history" && Boolean(profileId),
  });
  const highlights = useQuery({
    queryKey: queryKeys.articleHighlights(profileId),
    queryFn: () => desktop.listArticleHighlights(500),
    enabled: kind === "highlights" && Boolean(profileId),
  });
  const active = kind === "history" ? history : highlights;

  const deleteHighlight = useMutation({
    mutationFn: desktop.deleteArticleHighlight,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.articleHighlights(profileId) });
      void queryClient.invalidateQueries({ queryKey: queryKeys.articles(profileId) });
    },
    onError: (mutationError) => setError(errorMessage(mutationError)),
  });

  return (
    <div className="bergag-activity-view" data-testid={`article-${kind}`}>
      <header className="bergag-activity-view__header">
        <p className="bergag-eyebrow">Reading activity</p>
        <h1 className="bergag-title bergag-title--small">
          {kind === "history" ? "Reading history" : "Saved highlights"}
        </h1>
        <p className="bergag-subtitle">
          {kind === "history"
            ? "Local events recorded when articles were opened or their read state changed."
            : "Text you selected while reading. Nothing is sent anywhere."}
        </p>
      </header>

      {error ? (
        <div className="bergag-error" role="alert">
          {error}
        </div>
      ) : null}
      {active.isLoading ? <div className="bergag-loading">Loading activity…</div> : null}
      {active.error ? (
        <div className="bergag-error" role="alert">
          {errorMessage(active.error)}
        </div>
      ) : null}

      {kind === "history" && history.data?.length === 0 ? (
        <div className="bergag-empty-state">
          <h2 className="bergag-empty-state__title">No history recorded</h2>
          <p className="bergag-empty-state__text">
            Open an article to create a local history entry.
          </p>
        </div>
      ) : null}

      {kind === "history"
        ? history.data?.map((entry) => (
            <article className="bergag-activity-row" key={entry.id}>
              <div>
                <div className="bergag-activity-row__title">{entry.title}</div>
                <div className="bergag-activity-row__meta">
                  {entry.feed_title} · {entry.event_type} · {formatArticleDate(entry.occurred_at)}
                </div>
              </div>
              <Button variant="ghost" size="sm" onClick={() => onSelectArticle(entry.item_id)}>
                Open article
              </Button>
            </article>
          ))
        : null}

      {kind === "highlights" && highlights.data?.length === 0 ? (
        <div className="bergag-empty-state">
          <h2 className="bergag-empty-state__title">No highlights saved</h2>
          <p className="bergag-empty-state__text">
            Select text inside an article and choose “Save selected text as highlight.”
          </p>
        </div>
      ) : null}

      {kind === "highlights"
        ? highlights.data?.map((entry) => (
            <article className="bergag-activity-row" key={entry.id}>
              <div>
                <blockquote className="bergag-activity-row__quote">“{entry.text}”</blockquote>
                <div className="bergag-activity-row__meta">
                  {entry.article_title} · {entry.feed_title} · {formatArticleDate(entry.created_at)}
                </div>
              </div>
              <div className="bergag-inline-actions">
                <Button variant="ghost" size="sm" onClick={() => onSelectArticle(entry.item_id)}>
                  Open article
                </Button>
                <IconButton
                  label="Delete highlight"
                  icon={<Trash2 size={13} aria-hidden="true" />}
                  disabled={deleteHighlight.isPending}
                  onClick={() => deleteHighlight.mutate(entry.id)}
                />
              </div>
            </article>
          ))
        : null}
    </div>
  );
}
