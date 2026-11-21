import * as Dialog from "@radix-ui/react-dialog";
import {
  isValidWidgetTitle,
  usesDayRange,
  usesItemLimit,
  widgetKindLabel,
} from "../../application/dashboardService";
import { Button } from "../../components/ui/Button";
import { IconButton } from "../../components/ui/IconButton";
import type { DashboardWidget } from "../../domain/models";
import { LoaderCircle, X } from "lucide-react";
import { useState } from "react";

export interface WidgetEditorValue {
  title: string;
  days: number;
  itemLimit: number;
  sortOrder: number;
}

interface WidgetEditorDialogProps {
  open: boolean;
  widget: DashboardWidget | null;
  saving: boolean;
  onOpenChange: (open: boolean) => void;
  onSave: (value: WidgetEditorValue) => void;
}

export function WidgetEditorDialog({
  open,
  widget,
  saving,
  onOpenChange,
  onSave,
}: WidgetEditorDialogProps) {
  const [title, setTitle] = useState(widget?.title ?? "");
  const [days, setDays] = useState(widget?.days ?? 30);
  const [itemLimit, setItemLimit] = useState(widget?.item_limit ?? 5);
  const [sortOrder, setSortOrder] = useState(widget?.sort_order ?? 0);
  const showDays = widget ? usesDayRange(widget.kind) : false;
  const showItemLimit = widget ? usesItemLimit(widget.kind) : false;

  const valid =
    isValidWidgetTitle(title) &&
    days >= 1 &&
    days <= 365 &&
    itemLimit >= 1 &&
    itemLimit <= 20 &&
    sortOrder >= 0 &&
    sortOrder <= 1000;

  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay className="bergag-dialog-overlay" />
        <Dialog.Content className="bergag-dialog bergag-dialog--compact">
          <form
            className="bergag-dialog__body"
            onSubmit={(event) => {
              event.preventDefault();
              if (!valid || saving) return;
              onSave({ title: title.trim(), days, itemLimit, sortOrder });
            }}
          >
            <div className="bergag-dialog__header">
              <div>
                <Dialog.Title className="bergag-title bergag-title--small">
                  Edit widget
                </Dialog.Title>
                <Dialog.Description className="bergag-subtitle">
                  {widget ? widgetKindLabel(widget.kind) : "Widget"}
                </Dialog.Description>
              </div>
              <Dialog.Close asChild>
                <IconButton label="Close widget editor" icon={<X size={15} />} />
              </Dialog.Close>
            </div>
            <div className="bergag-dialog__content">
              <label className="bergag-field">
                <span className="bergag-field__label">Title</span>
                <input
                  className="bergag-input"
                  value={title}
                  maxLength={96}
                  onChange={(event) => setTitle(event.target.value)}
                />
                <span className="bergag-field__hint">Shown on the dashboard card.</span>
              </label>
              <div className="bergag-dashboard-form-grid">
                {showDays ? (
                  <label className="bergag-field">
                    <span className="bergag-field__label">
                      {widget?.kind === "briefing" ? "Briefing period (days)" : "Days"}
                    </span>
                    <input
                      className="bergag-input"
                      type="number"
                      min={1}
                      max={365}
                      value={days}
                      onChange={(event) => setDays(Number(event.target.value))}
                    />
                    <span className="bergag-field__hint">
                      {widget?.kind === "briefing"
                        ? "1 uses a daily briefing; 2 or more uses a weekly briefing."
                        : "1 to 365 days of local history."}
                    </span>
                  </label>
                ) : null}
                {showItemLimit ? (
                  <label className="bergag-field">
                    <span className="bergag-field__label">Item limit</span>
                    <input
                      className="bergag-input"
                      type="number"
                      min={1}
                      max={20}
                      value={itemLimit}
                      onChange={(event) => setItemLimit(Number(event.target.value))}
                    />
                    <span className="bergag-field__hint">1 to 20 items.</span>
                  </label>
                ) : null}
                <label className="bergag-field">
                  <span className="bergag-field__label">Position</span>
                  <input
                    className="bergag-input"
                    type="number"
                    min={0}
                    max={1000}
                    value={sortOrder}
                    onChange={(event) => setSortOrder(Number(event.target.value))}
                  />
                  <span className="bergag-field__hint">Lower numbers appear first.</span>
                </label>
              </div>
              {!showDays && !showItemLimit ? (
                <div className="bergag-note">
                  This widget has no range or item limit to configure.
                </div>
              ) : null}
            </div>
            <div className="bergag-dashboard-editor-actions">
              <Dialog.Close asChild>
                <Button variant="ghost">Cancel</Button>
              </Dialog.Close>
              <Button type="submit" variant="primary" disabled={!valid || saving}>
                {saving ? <LoaderCircle size={14} className="bergag-spin" /> : null}
                Save widget
              </Button>
            </div>
          </form>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
