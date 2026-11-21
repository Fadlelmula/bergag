import * as Dialog from "@radix-ui/react-dialog";
import { LoaderCircle, Plus, Search, X } from "lucide-react";
import { useCallback, useState } from "react";
import { feedTypeLabel, prepareFeedUrl } from "../../application/feedService";
import { Button } from "../../components/ui/Button";
import { IconButton } from "../../components/ui/IconButton";
import type { DiscoveredFeed, FeedFolder, FeedPreview, SubscribeResult } from "../../domain/models";
import { desktop } from "../../infrastructure/tauri/commands";

interface AddFeedDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  folders: FeedFolder[];
  onSubscribed: (result: SubscribeResult) => void;
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export function AddFeedDialog({ open, onOpenChange, folders, onSubscribed }: AddFeedDialogProps) {
  const [url, setUrl] = useState("");
  const [folderId, setFolderId] = useState("");
  const [preview, setPreview] = useState<FeedPreview | null>(null);
  const [discovered, setDiscovered] = useState<DiscoveredFeed[] | null>(null);
  const [busy, setBusy] = useState<"preview" | "discover" | "subscribe" | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  const reset = useCallback(() => {
    setUrl("");
    setFolderId("");
    setPreview(null);
    setDiscovered(null);
    setBusy(null);
    setError(null);
    setNotice(null);
  }, []);

  const changeOpen = (next: boolean) => {
    if (!next) reset();
    onOpenChange(next);
  };

  const runPreview = async (target = url) => {
    const prepared = prepareFeedUrl(target);
    if (!prepared) {
      setError("Enter a feed URL first.");
      return;
    }
    setBusy("preview");
    setError(null);
    setNotice(null);
    setPreview(null);
    try {
      const result = await desktop.previewFeed(prepared);
      setUrl(prepared);
      setPreview(result);
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  };

  const runDiscover = async () => {
    const prepared = prepareFeedUrl(url);
    if (!prepared) {
      setError("Enter a website URL first.");
      return;
    }
    setBusy("discover");
    setError(null);
    setNotice(null);
    setDiscovered(null);
    try {
      const result = await desktop.discoverFeeds(prepared);
      setDiscovered(result);
      if (result.length === 0) setNotice("No feed links were found on that page.");
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  };

  const subscribe = async () => {
    const prepared = prepareFeedUrl(url);
    if (!prepared) {
      setError("Enter a feed URL first.");
      return;
    }
    setBusy("subscribe");
    setError(null);
    setNotice(null);
    try {
      const result = await desktop.subscribeFeed(prepared, folderId || null);
      onSubscribed(result);
      if (result.duplicate) {
        setNotice("This feed is already subscribed. Nothing was changed.");
        setBusy(null);
        return;
      }
      changeOpen(false);
    } catch (caught) {
      setError(errorMessage(caught));
      setBusy(null);
    }
  };

  return (
    <Dialog.Root open={open} onOpenChange={changeOpen}>
      <Dialog.Portal>
        <Dialog.Overlay className="bergag-dialog-overlay" />
        <Dialog.Content className="bergag-dialog" aria-describedby="add-feed-description">
          <div className="bergag-dialog__body">
            <div className="bergag-dialog__header">
              <div>
                <Dialog.Title className="bergag-title bergag-title--small">Add a feed</Dialog.Title>
                <Dialog.Description id="add-feed-description" className="bergag-subtitle">
                  Paste a feed URL, or scan a website for its feed links. Feeds are fetched directly
                  by Bergag; no third-party service is used.
                </Dialog.Description>
              </div>
              <Dialog.Close asChild>
                <IconButton label="Close add feed" icon={<X size={15} aria-hidden="true" />} />
              </Dialog.Close>
            </div>
            <div className="bergag-dialog__content" style={{ maxWidth: 720 }}>
              <div className="bergag-field">
                <label className="bergag-field__label" htmlFor="feed-url">
                  Feed or website URL
                </label>
                <input
                  id="feed-url"
                  className="bergag-input"
                  type="url"
                  inputMode="url"
                  autoComplete="off"
                  placeholder="https://example.com/feed.xml"
                  value={url}
                  onChange={(event) => setUrl(event.target.value)}
                  onKeyDown={(event) => {
                    if (event.key === "Enter") void runPreview();
                  }}
                />
                <span className="bergag-field__hint">
                  Only public http and https addresses are accepted. Private network addresses are
                  blocked.
                </span>
              </div>

              <div className="bergag-inline-actions" style={{ marginBottom: 18 }}>
                <Button
                  variant="secondary"
                  onClick={() => void runPreview()}
                  disabled={busy !== null}
                  icon={busy === "preview" ? <LoaderCircle size={14} /> : <Plus size={14} />}
                >
                  Preview feed
                </Button>
                <Button
                  variant="ghost"
                  onClick={() => void runDiscover()}
                  disabled={busy !== null}
                  icon={busy === "discover" ? <LoaderCircle size={14} /> : <Search size={14} />}
                >
                  Find feeds on page
                </Button>
              </div>

              {error ? <p className="bergag-error">{error}</p> : null}
              {notice ? <p className="bergag-field__hint">{notice}</p> : null}

              {discovered && discovered.length > 0 ? (
                <div className="bergag-feed-results">
                  <div className="bergag-section-title" style={{ marginTop: 8 }}>
                    Found feeds
                  </div>
                  {discovered.map((candidate) => (
                    <button
                      key={candidate.url}
                      type="button"
                      className="bergag-feed-row"
                      onClick={() => {
                        setUrl(candidate.url);
                        void runPreview(candidate.url);
                      }}
                    >
                      <span className="bergag-feed-row__title">{candidate.title}</span>
                      <span className="bergag-feed-row__meta">{candidate.url}</span>
                    </button>
                  ))}
                </div>
              ) : null}

              {preview ? (
                <div className="bergag-card bergag-card--padded" style={{ marginTop: 16 }}>
                  <div className="bergag-stat__label">Preview</div>
                  <div className="bergag-stat__value">{preview.title}</div>
                  <div className="bergag-stat__detail">
                    {feedTypeLabel(preview.feed_type)} · {preview.item_count} item
                    {preview.item_count === 1 ? "" : "s"}
                    {preview.description ? ` · ${preview.description}` : ""}
                  </div>
                  {preview.items.length > 0 ? (
                    <ul className="bergag-bullet-list" style={{ marginTop: 12 }}>
                      {preview.items.slice(0, 5).map((item) => (
                        <li key={item.guid}>{item.title}</li>
                      ))}
                    </ul>
                  ) : null}
                </div>
              ) : null}

              <div className="bergag-field" style={{ marginTop: 18 }}>
                <label className="bergag-field__label" htmlFor="feed-folder">
                  Folder (optional)
                </label>
                <select
                  id="feed-folder"
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

              <div className="bergag-inline-actions">
                <Button
                  variant="primary"
                  onClick={() => void subscribe()}
                  disabled={busy !== null || !url.trim()}
                  icon={busy === "subscribe" ? <LoaderCircle size={14} /> : <Plus size={14} />}
                >
                  Subscribe
                </Button>
              </div>
            </div>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
