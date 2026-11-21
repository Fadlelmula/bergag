import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  normalizeDashboardName,
  sortWidgets,
  WIDGET_KINDS,
  type WidgetKind,
} from "../../application/dashboardService";
import { errorMessage } from "../../application/articleService";
import { queryKeys } from "../../application/queryKeys";
import { Button } from "../../components/ui/Button";
import { IconButton } from "../../components/ui/IconButton";
import type { DashboardWidget } from "../../domain/models";
import { desktop } from "../../infrastructure/tauri/commands";
import { LayoutDashboard, LoaderCircle, Pencil, Plus, Star, Trash2 } from "lucide-react";
import { useState } from "react";
import { AnalyticsEmpty } from "./AnalyticsBits";
import { DashboardWidgetCard } from "./DashboardWidgetCard";
import { WidgetEditorDialog, type WidgetEditorValue } from "./WidgetEditorDialog";

interface DashboardPanelProps {
  profileId: string;
  onOpenArticle: (id: string) => void;
}

interface DashboardForm {
  id: string | null;
  name: string;
  description: string;
}

export function DashboardPanel({ profileId, onOpenArticle }: DashboardPanelProps) {
  const queryClient = useQueryClient();
  const dashboards = useQuery({
    queryKey: queryKeys.dashboards(profileId),
    queryFn: desktop.listDashboards,
    enabled: Boolean(profileId),
  });
  const [requestedId, setRequestedId] = useState<string | null>(null);
  const [formOverride, setFormOverride] = useState<DashboardForm | null>(null);
  const [newKind, setNewKind] = useState<WidgetKind>("unread_by_feed");
  const [editingWidget, setEditingWidget] = useState<DashboardWidget | null>(null);
  const [editorOpen, setEditorOpen] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);

  const list = dashboards.data ?? [];
  const defaultId = list.find((dashboard) => dashboard.is_default)?.id ?? list[0]?.id ?? null;
  const selectedId =
    requestedId && list.some((dashboard) => dashboard.id === requestedId) ? requestedId : defaultId;
  const selected = list.find((dashboard) => dashboard.id === selectedId);
  const form: DashboardForm =
    formOverride && (formOverride.id === null || formOverride.id === selectedId)
      ? formOverride
      : selected
        ? {
            id: selected.id,
            name: selected.name,
            description: selected.description ?? "",
          }
        : { id: null, name: "", description: "" };

  const widgets = useQuery({
    queryKey: queryKeys.dashboardWidgets(profileId, selectedId ?? "none"),
    queryFn: () => desktop.listWidgets(selectedId ?? ""),
    enabled: Boolean(profileId && selectedId),
  });

  const invalidateDashboards = async () => {
    await queryClient.invalidateQueries({ queryKey: queryKeys.dashboards(profileId) });
  };

  const invalidateWidgets = async (dashboardId: string) => {
    await queryClient.invalidateQueries({
      queryKey: queryKeys.dashboardWidgets(profileId, dashboardId),
    });
    await queryClient.invalidateQueries({ queryKey: queryKeys.dashboards(profileId) });
  };

  const saveDashboard = useMutation({
    mutationFn: () =>
      desktop.saveDashboard(
        form.id,
        normalizeDashboardName(form.name),
        form.description.trim() || null,
      ),
    onSuccess: async (dashboard) => {
      setRequestedId(dashboard.id);
      setFormOverride(null);
      setNotice(`Saved “${dashboard.name}”.`);
      setActionError(null);
      await invalidateDashboards();
    },
    onError: (error) => setActionError(errorMessage(error)),
  });

  const deleteDashboard = useMutation({
    mutationFn: (id: string) => desktop.deleteDashboard(id),
    onSuccess: async () => {
      setRequestedId(null);
      setFormOverride(null);
      setNotice("Dashboard deleted. Articles and other data were not changed.");
      setActionError(null);
      await invalidateDashboards();
    },
    onError: (error) => setActionError(errorMessage(error)),
  });

  const setDefault = useMutation({
    mutationFn: (id: string) => desktop.setDefaultDashboard(id),
    onSuccess: async (dashboard) => {
      setNotice(`“${dashboard.name}” is now the default.`);
      setActionError(null);
      await invalidateDashboards();
    },
    onError: (error) => setActionError(errorMessage(error)),
  });

  const addWidget = useMutation({
    mutationFn: (kind: WidgetKind) => desktop.addWidget(selectedId ?? "", kind),
    onSuccess: async (widget) => {
      setEditingWidget(widget);
      setEditorOpen(true);
      setNotice("Widget added. Adjust its settings below.");
      setActionError(null);
      await invalidateWidgets(widget.dashboard_id);
    },
    onError: (error) => setActionError(errorMessage(error)),
  });

  const updateWidget = useMutation({
    mutationFn: ({ widget, value }: { widget: DashboardWidget; value: WidgetEditorValue }) =>
      desktop.updateWidget(widget.id, value.title, value.days, value.itemLimit, value.sortOrder),
    onSuccess: async () => {
      setEditorOpen(false);
      setEditingWidget(null);
      setNotice("Widget settings saved.");
      setActionError(null);
      if (selectedId) await invalidateWidgets(selectedId);
    },
    onError: (error) => setActionError(errorMessage(error)),
  });

  const deleteWidget = useMutation({
    mutationFn: (id: string) => desktop.deleteWidget(id),
    onSuccess: async () => {
      setNotice("Widget removed from the dashboard.");
      setActionError(null);
      if (selectedId) await invalidateWidgets(selectedId);
    },
    onError: (error) => setActionError(errorMessage(error)),
  });

  const orderedWidgets = sortWidgets(widgets.data ?? []);

  return (
    <div className="bergag-dashboard-panel">
      <div className="bergag-dashboard-toolbar">
        <div className="bergag-dashboard-select">
          <label className="bergag-field__label" htmlFor="dashboard-select">
            Dashboard
          </label>
          <select
            id="dashboard-select"
            className="bergag-select"
            value={selectedId ?? ""}
            onChange={(event) => setRequestedId(event.target.value || null)}
            disabled={(dashboards.data ?? []).length === 0}
          >
            {(dashboards.data ?? []).length === 0 ? (
              <option value="">No dashboards yet</option>
            ) : (
              (dashboards.data ?? []).map((dashboard) => (
                <option key={dashboard.id} value={dashboard.id}>
                  {dashboard.name}
                  {dashboard.is_default ? " (default)" : ""}
                </option>
              ))
            )}
          </select>
        </div>
        <div className="bergag-inline-actions">
          <Button
            variant="ghost"
            icon={<Plus size={14} />}
            onClick={() => setFormOverride({ id: null, name: "", description: "" })}
          >
            New dashboard
          </Button>
          <Button
            variant="secondary"
            icon={<Star size={14} />}
            disabled={!selected || selected.is_default || setDefault.isPending}
            onClick={() => selected && setDefault.mutate(selected.id)}
          >
            Make default
          </Button>
          <Button
            variant="danger"
            icon={<Trash2 size={14} />}
            disabled={!selected || deleteDashboard.isPending}
            onClick={() => {
              if (!selected) return;
              if (
                window.confirm(
                  `Delete “${selected.name}” and its widget settings? Articles and reading data stay in the library.`,
                )
              ) {
                deleteDashboard.mutate(selected.id);
              }
            }}
          >
            Delete
          </Button>
        </div>
      </div>

      <div className="bergag-card bergag-card--padded bergag-dashboard-form">
        <div className="bergag-dashboard-form-grid">
          <label className="bergag-field">
            <span className="bergag-field__label">
              {form.id ? "Rename dashboard" : "New dashboard"}
            </span>
            <input
              className="bergag-input"
              value={form.name}
              maxLength={64}
              placeholder="Daily overview"
              onChange={(event) => setFormOverride({ ...form, name: event.target.value })}
            />
            <span className="bergag-field__hint">Up to 64 characters; unique per profile.</span>
          </label>
          <label className="bergag-field">
            <span className="bergag-field__label">Description</span>
            <input
              className="bergag-input"
              value={form.description}
              maxLength={240}
              placeholder="Optional local note"
              onChange={(event) => setFormOverride({ ...form, description: event.target.value })}
            />
            <span className="bergag-field__hint">Optional, stored only in this profile.</span>
          </label>
        </div>
        <div className="bergag-inline-actions">
          <Button
            variant="primary"
            disabled={!normalizeDashboardName(form.name) || saveDashboard.isPending}
            icon={
              saveDashboard.isPending ? <LoaderCircle size={14} /> : <LayoutDashboard size={14} />
            }
            onClick={() => saveDashboard.mutate()}
          >
            {form.id ? "Save dashboard" : "Create dashboard"}
          </Button>
        </div>
      </div>

      <div className="bergag-dashboard-widget-toolbar">
        <div>
          <h3 className="bergag-section-title">Widgets</h3>
          <p className="bergag-subtitle">
            Add only the widgets you want. Each card reads local data when opened.
          </p>
        </div>
        <div className="bergag-inline-actions">
          <select
            className="bergag-select"
            aria-label="Widget type"
            value={newKind}
            onChange={(event) => setNewKind(event.target.value as WidgetKind)}
          >
            {WIDGET_KINDS.map((option) => (
              <option key={option.kind} value={option.kind}>
                {option.label}
              </option>
            ))}
          </select>
          <Button
            variant="secondary"
            icon={<Plus size={14} />}
            disabled={!selectedId || addWidget.isPending}
            onClick={() => addWidget.mutate(newKind)}
          >
            Add widget
          </Button>
        </div>
      </div>

      {dashboards.isLoading ? <div className="bergag-loading">Loading dashboards…</div> : null}
      {dashboards.isError ? (
        <div className="bergag-error" role="alert">
          {errorMessage(dashboards.error)}
        </div>
      ) : null}
      {!dashboards.isLoading && (dashboards.data ?? []).length === 0 ? (
        <AnalyticsEmpty
          title="No dashboards yet"
          text="Create a dashboard, then add widgets that read only this profile's local data."
        />
      ) : null}
      {selectedId && widgets.isLoading ? (
        <div className="bergag-loading">Loading widgets…</div>
      ) : null}
      {selectedId && widgets.isError ? (
        <div className="bergag-error" role="alert">
          {errorMessage(widgets.error)}
        </div>
      ) : null}
      {selectedId && !widgets.isLoading && orderedWidgets.length === 0 ? (
        <AnalyticsEmpty
          title="This dashboard is empty"
          text="Choose a widget type above and add it to the dashboard."
        />
      ) : null}
      {selectedId && orderedWidgets.length > 0 ? (
        <div className="bergag-widget-grid">
          {orderedWidgets.map((widget) => (
            <div className="bergag-widget-wrap" key={widget.id}>
              <DashboardWidgetCard
                profileId={profileId}
                widget={widget}
                onOpenArticle={onOpenArticle}
              />
              <div className="bergag-widget-actions">
                <IconButton
                  label={`Edit ${widget.title}`}
                  icon={<Pencil size={14} />}
                  onClick={() => {
                    setEditingWidget(widget);
                    setEditorOpen(true);
                  }}
                />
                <IconButton
                  label={`Remove ${widget.title}`}
                  icon={<Trash2 size={14} />}
                  onClick={() => {
                    if (window.confirm(`Remove “${widget.title}” from this dashboard?`)) {
                      deleteWidget.mutate(widget.id);
                    }
                  }}
                />
              </div>
            </div>
          ))}
        </div>
      ) : null}

      {notice ? <div className="bergag-note">{notice}</div> : null}
      {actionError ? (
        <div className="bergag-error" role="alert">
          {actionError}
        </div>
      ) : null}

      <WidgetEditorDialog
        key={editingWidget?.id ?? "none"}
        open={editorOpen}
        widget={editingWidget}
        saving={updateWidget.isPending}
        onOpenChange={(open) => {
          setEditorOpen(open);
          if (!open) setEditingWidget(null);
        }}
        onSave={(value) => {
          if (!editingWidget) return;
          updateWidget.mutate({ widget: editingWidget, value });
        }}
      />
    </div>
  );
}
