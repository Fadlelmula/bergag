import * as Dialog from "@radix-ui/react-dialog";
import { LoaderCircle, X } from "lucide-react";
import { useState } from "react";
import { refreshIntervalLabel } from "../../application/feedService";
import { Button } from "../../components/ui/Button";
import { IconButton } from "../../components/ui/IconButton";
import type { Feed, FeedFolder, TrustLevel } from "../../domain/models";
import { desktop } from "../../infrastructure/tauri/commands";

interface FeedSettingsDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  mode: "folder" | "feed";
  feed: Feed | null;
  folders: FeedFolder[];
  onSaved: () => void;
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

const INTERVALS = [5, 15, 30, 60, 180, 360, 720, 1440, 10080];

export function FeedSettingsDialog({
  open,
  onOpenChange,
  mode,
  feed,
  folders,
  onSaved,
}: FeedSettingsDialogProps) {
  const [name, setName] = useState("");
  const [title, setTitle] = useState(feed?.title ?? "");
  const [folderId, setFolderId] = useState(feed?.folder_id ?? "");
  const [status, setStatus] = useState<"active" | "paused">(feed?.status ?? "active");
  const [interval, setInterval] = useState(feed?.refresh_interval_minutes ?? 60);
  const [category, setCategory] = useState(feed?.category ?? "");
  const [trustLevel, setTrustLevel] = useState<TrustLevel>(feed?.trust_level ?? "untrusted");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const save = async () => {
    setBusy(true);
    setError(null);
    try {
      if (mode === "folder") {
        await desktop.createFeedFolder(name);
      } else if (feed) {
        await desktop.updateFeed(
          feed.id,
          title,
          folderId || null,
          status,
          interval,
          category.trim() || null,
          trustLevel,
        );
      }
      onSaved();
      onOpenChange(false);
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(false);
    }
  };

  const isFolder = mode === "folder";

  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay className="bergag-dialog-overlay" />
        <Dialog.Content
          className="bergag-dialog"
          style={{ width: "min(520px, calc(100vw - 48px))", height: "auto" }}
        >
          <div className="bergag-dialog__body">
            <div className="bergag-dialog__header">
              <div>
                <Dialog.Title className="bergag-title bergag-title--small">
                  {isFolder ? "New folder" : "Feed settings"}
                </Dialog.Title>
                <Dialog.Description className="bergag-subtitle">
                  {isFolder
                    ? "Group related feeds. Folder names must be unique within this profile."
                    : "Change how this feed is named, grouped, and refreshed."}
                </Dialog.Description>
              </div>
              <Dialog.Close asChild>
                <IconButton label="Close" icon={<X size={15} aria-hidden="true" />} />
              </Dialog.Close>
            </div>
            <div className="bergag-dialog__content">
              {isFolder ? (
                <div className="bergag-field">
                  <label className="bergag-field__label" htmlFor="folder-name">
                    Folder name
                  </label>
                  <input
                    id="folder-name"
                    className="bergag-input"
                    value={name}
                    maxLength={64}
                    onChange={(event) => setName(event.target.value)}
                    autoFocus
                  />
                </div>
              ) : (
                <>
                  <div className="bergag-field">
                    <label className="bergag-field__label" htmlFor="edit-feed-title">
                      Display title
                    </label>
                    <input
                      id="edit-feed-title"
                      className="bergag-input"
                      value={title}
                      maxLength={256}
                      onChange={(event) => setTitle(event.target.value)}
                    />
                  </div>
                  <div className="bergag-field">
                    <label className="bergag-field__label" htmlFor="edit-feed-folder">
                      Folder
                    </label>
                    <select
                      id="edit-feed-folder"
                      className="bergag-select"
                      value={folderId}
                      onChange={(event) => setFolderId(event.target.value)}
                    >
                      <option value="">No folder</option>
                      {folders.map((folder) => (
                        <option key={folder.id} value={folder.id}>
                          {folder.name}
                        </option>
                      ))}
                    </select>
                  </div>
                  <div className="bergag-field">
                    <label className="bergag-field__label" htmlFor="edit-feed-category">
                      Category
                    </label>
                    <input
                      id="edit-feed-category"
                      className="bergag-input"
                      value={category}
                      maxLength={64}
                      placeholder="e.g. Security, Design, Local news"
                      onChange={(event) => setCategory(event.target.value)}
                    />
                    <span className="bergag-field__hint">
                      Categories group feeds and can be used in rules.
                    </span>
                  </div>
                  <div className="bergag-field">
                    <label className="bergag-field__label" htmlFor="edit-feed-trust">
                      Trust level
                    </label>
                    <select
                      id="edit-feed-trust"
                      className="bergag-select"
                      value={trustLevel}
                      onChange={(event) => setTrustLevel(event.target.value as TrustLevel)}
                    >
                      <option value="untrusted">Untrusted</option>
                      <option value="low">Low</option>
                      <option value="medium">Medium</option>
                      <option value="high">High</option>
                    </select>
                    <span className="bergag-field__hint">
                      Local metadata only. Trust never changes what is fetched automatically.
                    </span>
                  </div>
                  <div className="bergag-field">
                    <label className="bergag-field__label" htmlFor="edit-feed-status">
                      Status
                    </label>
                    <select
                      id="edit-feed-status"
                      className="bergag-select"
                      value={status}
                      onChange={(event) => setStatus(event.target.value as "active" | "paused")}
                    >
                      <option value="active">Active</option>
                      <option value="paused">Paused</option>
                    </select>
                  </div>
                  <div className="bergag-field">
                    <label className="bergag-field__label" htmlFor="edit-feed-interval">
                      Refresh interval
                    </label>
                    <select
                      id="edit-feed-interval"
                      className="bergag-select"
                      value={interval}
                      onChange={(event) => setInterval(Number(event.target.value))}
                    >
                      {INTERVALS.map((value) => (
                        <option key={value} value={value}>
                          {refreshIntervalLabel(value)}
                        </option>
                      ))}
                      {!INTERVALS.includes(interval) ? (
                        <option value={interval}>{refreshIntervalLabel(interval)}</option>
                      ) : null}
                    </select>
                    <span className="bergag-field__hint">
                      Failed refreshes back off automatically. Successful refreshes restore this
                      interval.
                    </span>
                  </div>
                </>
              )}

              {error ? <p className="bergag-error">{error}</p> : null}
              <div className="bergag-inline-actions">
                <Button
                  variant="primary"
                  onClick={() => void save()}
                  disabled={busy || (isFolder ? !name.trim() : !title.trim())}
                  icon={busy ? <LoaderCircle size={14} /> : undefined}
                >
                  Save
                </Button>
                <Dialog.Close asChild>
                  <Button variant="ghost">Cancel</Button>
                </Dialog.Close>
              </div>
            </div>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
