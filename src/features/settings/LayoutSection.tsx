import { useLayout } from "../../app/LayoutProvider";
import { Button } from "../../components/ui/Button";
import { SwitchField } from "../../components/ui/SwitchField";
import { DEFAULT_LAYOUT } from "../../domain/layout";
import { useState } from "react";

export function LayoutSection() {
  const { layout, updateLayout, resetLayout, isSaving, error } = useLayout();
  const [confirmReset, setConfirmReset] = useState(false);
  const [localError, setLocalError] = useState<string | null>(null);

  const visibleCount = [
    layout.sidebar.visible,
    layout.panels.articleList.visible,
    layout.panels.readingPane.visible,
  ].filter(Boolean).length;

  const doReset = async () => {
    setLocalError(null);
    try {
      await resetLayout();
      setConfirmReset(false);
    } catch (resetError) {
      setLocalError(
        resetError instanceof Error ? resetError.message : String(resetError ?? "Unknown error"),
      );
    }
  };

  return (
    <div>
      <p className="bergag-eyebrow">Layout</p>
      <h2 className="bergag-title bergag-title--small">Multi-panel workspace</h2>
      <p className="bergag-subtitle">
        Panel visibility, widths, sidebar order, and density are saved automatically to this
        profile. Drag panel dividers to resize; right-click sidebar items to reorder or hide them.
      </p>

      <div style={{ marginTop: 18 }}>
        <SwitchField
          label="Show sidebar"
          checked={layout.sidebar.visible}
          disabled={layout.sidebar.visible && visibleCount <= 1}
          onCheckedChange={(checked) =>
            updateLayout((current) => ({
              ...current,
              sidebar: { ...current.sidebar, visible: checked },
            }))
          }
        />
        <SwitchField
          label="Show article list"
          checked={layout.panels.articleList.visible}
          disabled={layout.panels.articleList.visible && visibleCount <= 1}
          onCheckedChange={(checked) =>
            updateLayout((current) => ({
              ...current,
              panels: {
                ...current.panels,
                articleList: { ...current.panels.articleList, visible: checked },
              },
            }))
          }
        />
        <SwitchField
          label="Show reading pane"
          checked={layout.panels.readingPane.visible}
          disabled={layout.panels.readingPane.visible && visibleCount <= 1}
          onCheckedChange={(checked) =>
            updateLayout((current) => ({
              ...current,
              panels: {
                ...current.panels,
                readingPane: { ...current.panels.readingPane, visible: checked },
              },
            }))
          }
        />
        <SwitchField
          label="Collapse sidebar"
          hint="Shows icons only while keeping sidebar actions available."
          checked={layout.sidebar.collapsed}
          onCheckedChange={(checked) =>
            updateLayout((current) => ({
              ...current,
              sidebar: { ...current.sidebar, collapsed: checked },
            }))
          }
        />
      </div>

      <div className="bergag-field" style={{ marginTop: 16 }}>
        <label className="bergag-field__label" htmlFor="sidebar-width">
          Sidebar width · {layout.sidebar.width}px
        </label>
        <input
          id="sidebar-width"
          className="bergag-range"
          type="range"
          min={200}
          max={420}
          value={layout.sidebar.width}
          onChange={(event) => {
            const width = Number(event.target.value);
            updateLayout((current) => ({ ...current, sidebar: { ...current.sidebar, width } }));
          }}
          onPointerUp={() => updateLayout((current) => current)}
        />
      </div>

      <div className="bergag-field">
        <label className="bergag-field__label" htmlFor="list-width">
          Article list width · {layout.panels.articleList.width}px
        </label>
        <input
          id="list-width"
          className="bergag-range"
          type="range"
          min={280}
          max={720}
          value={layout.panels.articleList.width}
          onChange={(event) => {
            const width = Number(event.target.value);
            updateLayout((current) => ({
              ...current,
              panels: {
                ...current.panels,
                articleList: { ...current.panels.articleList, width },
              },
            }));
          }}
          onPointerUp={() => updateLayout((current) => current)}
        />
      </div>

      <h3 className="bergag-section-title">Reset</h3>
      {!confirmReset ? (
        <div className="bergag-inline-actions">
          <Button
            variant="secondary"
            onClick={() =>
              updateLayout((current) => ({
                ...current,
                sidebar: {
                  ...current.sidebar,
                  order: [...DEFAULT_LAYOUT.sidebar.order],
                  hidden: [],
                },
              }))
            }
          >
            Reset sidebar order
          </Button>
          <Button variant="danger" onClick={() => setConfirmReset(true)}>
            Reset entire layout
          </Button>
        </div>
      ) : (
        <div className="bergag-inline-actions">
          <span className="bergag-key-description">Restore the default panel layout?</span>
          <Button variant="danger" onClick={() => void doReset()}>
            Confirm reset
          </Button>
          <Button variant="ghost" onClick={() => setConfirmReset(false)}>
            Cancel
          </Button>
        </div>
      )}

      {isSaving ? <p className="bergag-field__hint">Saving layout…</p> : null}
      {(localError || error) && (
        <p className="bergag-error" role="alert">
          {localError ?? error}
        </p>
      )}
    </div>
  );
}
