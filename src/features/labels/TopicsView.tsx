import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { LoaderCircle, Plus, Tag, Trash2 } from "lucide-react";
import { useState } from "react";
import { labelKindLabel } from "../../application/discoveryService";
import { queryKeys } from "../../application/queryKeys";
import { Button } from "../../components/ui/Button";
import type { Label, LabelKind } from "../../domain/models";
import { desktop } from "../../infrastructure/tauri/commands";

interface TopicsViewProps {
  profileId: string;
  onOpenArticle: (id: string) => void;
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export function TopicsView({ profileId, onOpenArticle }: TopicsViewProps) {
  const queryClient = useQueryClient();
  const labels = useQuery({
    queryKey: queryKeys.labels(profileId),
    queryFn: () => desktop.listLabels(null),
  });
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [kind, setKind] = useState<LabelKind>("tag");
  const [name, setName] = useState("");
  const [color, setColor] = useState("");
  const [description, setDescription] = useState("");
  const [notice, setNotice] = useState<string | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);

  const items = useQuery({
    queryKey: queryKeys.labelItems(profileId, selectedId ?? ""),
    queryFn: () => desktop.labelItems(selectedId ?? ""),
    enabled: Boolean(selectedId),
  });

  const selectedLabel = (labels.data ?? []).find((label) => label.id === selectedId) ?? null;

  const invalidate = async () => {
    await Promise.all([
      queryClient.invalidateQueries({ queryKey: queryKeys.labels(profileId) }),
      queryClient.invalidateQueries({ queryKey: ["articles"] }),
    ]);
    if (selectedId) {
      await queryClient.invalidateQueries({
        queryKey: queryKeys.labelItems(profileId, selectedId),
      });
    }
  };

  const save = useMutation({
    mutationFn: () => {
      if (selectedLabel) {
        return desktop.updateLabel(
          selectedLabel.id,
          name,
          color.trim() || null,
          description.trim() || null,
        );
      }
      return desktop.createLabel(kind, name, color.trim() || null, description.trim() || null);
    },
    onSuccess: async (label) => {
      setSelectedId(label.id);
      setKind(label.kind);
      setName(label.name);
      setColor(label.color ?? "");
      setDescription(label.description ?? "");
      setNotice(selectedLabel ? "Label updated." : "Label created.");
      setActionError(null);
      await invalidate();
    },
    onError: (error) => setActionError(errorMessage(error)),
  });

  const remove = useMutation({
    mutationFn: (id: string) => desktop.deleteLabel(id),
    onSuccess: async () => {
      setSelectedId(null);
      setName("");
      setColor("");
      setDescription("");
      setNotice("Label deleted. Article assignments for it were removed.");
      await invalidate();
    },
    onError: (error) => setActionError(errorMessage(error)),
  });

  const chooseLabel = (label: Label) => {
    setSelectedId(label.id);
    setKind(label.kind);
    setName(label.name);
    setColor(label.color ?? "");
    setDescription(label.description ?? "");
    setNotice(null);
  };

  const reset = () => {
    setSelectedId(null);
    setKind("tag");
    setName("");
    setColor("");
    setDescription("");
    setNotice(null);
  };

  const tags = (labels.data ?? []).filter((label) => label.kind === "tag");
  const topics = (labels.data ?? []).filter((label) => label.kind === "topic");

  return (
    <div className="bergag-full-view">
      <div className="bergag-full-view__header">
        <div>
          <h2 className="bergag-title">Topics & tags</h2>
          <p className="bergag-subtitle">
            Organize articles with private local labels. Tags and topics can also be used in
            automation rules.
          </p>
        </div>
        <Button variant="secondary" onClick={reset} icon={<Plus size={14} />}>
          New label
        </Button>
      </div>
      <div className="bergag-full-view__body">
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
        {labels.error ? (
          <div className="bergag-error" role="alert">
            {errorMessage(labels.error)}
          </div>
        ) : null}

        <div className="bergag-labels-grid">
          <section className="bergag-labels-column" aria-label="Labels">
            <div className="bergag-discovery-column__header">
              <h3 className="bergag-title bergag-title--small">
                {selectedLabel ? "Edit label" : "New label"}
              </h3>
              {selectedLabel ? (
                <Button variant="ghost" size="sm" onClick={reset}>
                  Cancel edit
                </Button>
              ) : null}
            </div>
            <div className="bergag-field">
              <label className="bergag-field__label" htmlFor="label-kind">
                Kind
              </label>
              <select
                id="label-kind"
                className="bergag-select"
                value={kind}
                disabled={Boolean(selectedLabel)}
                onChange={(event) => setKind(event.target.value as LabelKind)}
              >
                <option value="tag">Tag</option>
                <option value="topic">Topic</option>
              </select>
            </div>
            <div className="bergag-field">
              <label className="bergag-field__label" htmlFor="label-name">
                Name
              </label>
              <input
                id="label-name"
                className="bergag-input"
                value={name}
                maxLength={64}
                onChange={(event) => setName(event.target.value)}
              />
            </div>
            <div className="bergag-field">
              <label className="bergag-field__label" htmlFor="label-color">
                Color
              </label>
              <input
                id="label-color"
                className="bergag-input"
                value={color}
                maxLength={32}
                placeholder="Optional color token, e.g. amber"
                onChange={(event) => setColor(event.target.value)}
              />
            </div>
            <div className="bergag-field">
              <label className="bergag-field__label" htmlFor="label-description">
                Description
              </label>
              <input
                id="label-description"
                className="bergag-input"
                value={description}
                maxLength={500}
                onChange={(event) => setDescription(event.target.value)}
              />
            </div>
            <div className="bergag-inline-actions">
              <Button
                variant="primary"
                onClick={() => save.mutate()}
                disabled={!name.trim() || save.isPending}
                icon={save.isPending ? <LoaderCircle size={14} /> : <Plus size={14} />}
              >
                {selectedLabel ? "Save label" : "Create label"}
              </Button>
              {selectedLabel ? (
                <Button
                  variant="danger"
                  onClick={() => {
                    if (
                      window.confirm(
                        `Delete “${selectedLabel.name}”? Article assignments for this label will be removed.`,
                      )
                    ) {
                      remove.mutate(selectedLabel.id);
                    }
                  }}
                  disabled={remove.isPending}
                  icon={<Trash2 size={14} />}
                >
                  Delete
                </Button>
              ) : null}
            </div>
          </section>

          <section className="bergag-labels-column" aria-label="Label list">
            <div className="bergag-discovery-column__header">
              <h3 className="bergag-title bergag-title--small">Your labels</h3>
              <span className="bergag-feed-toolbar__count">
                {(labels.data ?? []).length} label{(labels.data ?? []).length === 1 ? "" : "s"}
              </span>
            </div>
            {labels.isLoading ? <div className="bergag-loading">Loading labels…</div> : null}
            {!labels.isLoading && (labels.data ?? []).length === 0 ? (
              <div className="bergag-empty-state">
                <div className="bergag-empty-state__title">No labels yet</div>
                <p className="bergag-empty-state__text">
                  Create a tag or topic, then assign it from an article reader.
                </p>
              </div>
            ) : null}
            {[
              { title: "Tags", items: tags },
              { title: "Topics", items: topics },
            ].map((group) =>
              group.items.length > 0 ? (
                <div className="bergag-label-group" key={group.title}>
                  <div className="bergag-nav-section">{group.title}</div>
                  {group.items.map((label) => (
                    <button
                      type="button"
                      key={label.id}
                      className={`bergag-label-row ${selectedId === label.id ? "bergag-label-row--active" : ""}`}
                      onClick={() => chooseLabel(label)}
                    >
                      <Tag size={13} aria-hidden="true" />
                      <span className="bergag-label-row__name">{label.name}</span>
                      {label.color ? <span className="bergag-badge">{label.color}</span> : null}
                      <span className="bergag-label-row__count">{label.usage_count}</span>
                    </button>
                  ))}
                </div>
              ) : null,
            )}
          </section>

          <section className="bergag-labels-column" aria-label="Label articles">
            <div className="bergag-discovery-column__header">
              <h3 className="bergag-title bergag-title--small">
                {selectedLabel ? `${selectedLabel.name} articles` : "Label articles"}
              </h3>
            </div>
            {!selectedLabel ? (
              <p className="bergag-empty-state__text">
                Select a label to see the articles assigned to it.
              </p>
            ) : items.isLoading ? (
              <div className="bergag-loading">Loading articles…</div>
            ) : (items.data ?? []).length === 0 ? (
              <p className="bergag-empty-state__text">
                No articles use {labelKindLabel(selectedLabel.kind).toLowerCase()} “
                {selectedLabel.name}” yet. Assign it from the article reader.
              </p>
            ) : (
              <div className="bergag-smart-feed-list">
                {(items.data ?? []).map((article) => (
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
            )}
          </section>
        </div>
      </div>
    </div>
  );
}
