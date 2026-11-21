import * as ContextMenu from "@radix-ui/react-context-menu";
import * as Dialog from "@radix-ui/react-dialog";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  Download,
  Folder,
  FolderPlus,
  MoreHorizontal,
  Plus,
  RefreshCw,
  Trash2,
  Upload,
  X,
} from "lucide-react";
import { useState } from "react";
import { feedTypeLabel, formatRelativeTime, healthLabel } from "../../application/feedService";
import { queryKeys } from "../../application/queryKeys";
import { Button } from "../../components/ui/Button";
import { IconButton } from "../../components/ui/IconButton";
import type { Feed, FeedFolder, SubscribeResult } from "../../domain/models";
import { desktop } from "../../infrastructure/tauri/commands";
import { AddFeedDialog } from "./AddFeedDialog";
import { FeedSettingsDialog } from "./FeedSettingsDialog";
import { ExportOpmlDialog, ImportOpmlDialog } from "./OpmlDialogs";

interface FeedsViewProps {
  profileId: string;
  selectedFeedId: string | null;
  onSelectFeed: (id: string) => void;
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export function FeedsView({ profileId, selectedFeedId, onSelectFeed }: FeedsViewProps) {
  const queryClient = useQueryClient();
  const folders = useQuery({
    queryKey: queryKeys.feedFolders(profileId),
    queryFn: desktop.listFeedFolders,
  });
  const feeds = useQuery({
    queryKey: queryKeys.feeds(profileId),
    queryFn: desktop.listFeeds,
  });
  const [addOpen, setAddOpen] = useState(false);
  const [settingsMode, setSettingsMode] = useState<"folder" | "feed" | null>(null);
  const [settingsFeed, setSettingsFeed] = useState<Feed | null>(null);
  const [importOpen, setImportOpen] = useState(false);
  const [exportOpen, setExportOpen] = useState(false);
  const [pendingDelete, setPendingDelete] = useState<Feed | null>(null);
  const [pendingFolderDelete, setPendingFolderDelete] = useState<FeedFolder | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);

  const invalidate = async () => {
    await Promise.all([
      queryClient.invalidateQueries({ queryKey: queryKeys.feeds(profileId) }),
      queryClient.invalidateQueries({ queryKey: queryKeys.feedFolders(profileId) }),
    ]);
  };

  const refresh = useMutation({
    mutationFn: (feedId: string | null) => desktop.refreshFeeds(feedId),
    onSuccess: async (outcomes) => {
      setNotice(
        outcomes.length === 0
          ? "No feeds were due for refresh."
          : `Refreshed ${outcomes.length} feed${outcomes.length === 1 ? "" : "s"}.`,
      );
      await invalidate();
      await queryClient.invalidateQueries({ queryKey: ["feeds", profileId, "items"] });
      await queryClient.invalidateQueries({ queryKey: ["feeds", profileId, "logs"] });
    },
    onError: (error) => setActionError(errorMessage(error)),
  });

  const remove = useMutation({
    mutationFn: (id: string) => desktop.deleteFeed(id),
    onSuccess: async () => {
      setPendingDelete(null);
      if (selectedFeedId === pendingDelete?.id) onSelectFirstAfterDelete();
      await invalidate();
    },
    onError: (error) => setActionError(errorMessage(error)),
  });

  const removeFolder = useMutation({
    mutationFn: (id: string) => desktop.deleteFeedFolder(id),
    onSuccess: async () => {
      setPendingFolderDelete(null);
      setNotice("Folder deleted. Its feeds were kept and moved to All feeds.");
      await invalidate();
    },
    onError: (error) => setActionError(errorMessage(error)),
  });

  const onSelectFirstAfterDelete = () => {
    const remaining = (feeds.data ?? []).filter((feed) => feed.id !== pendingDelete?.id);
    onSelectFeed(remaining[0]?.id ?? "");
  };

  const handleSubscribed = async (result: SubscribeResult) => {
    setNotice(
      result.duplicate
        ? "This feed is already subscribed. Nothing was changed."
        : `Subscribed to ${result.feed.title}.`,
    );
    await invalidate();
    if (!result.duplicate) onSelectFeed(result.feed.id);
  };

  const renderFeed = (feed: Feed) => (
    <ContextMenu.Root key={feed.id}>
      <ContextMenu.Trigger asChild>
        <button
          type="button"
          className={`bergag-feed-row ${selectedFeedId === feed.id ? "bergag-feed-row--active" : ""}`}
          onClick={() => onSelectFeed(feed.id)}
        >
          <span className={`bergag-health bergag-health--${feed.health}`} aria-hidden="true" />
          <span style={{ flex: 1, minWidth: 0 }}>
            <span className="bergag-feed-row__title">{feed.title}</span>
            <span className="bergag-feed-row__meta">
              {feedTypeLabel(feed.feed_type)} · {feed.article_count} item
              {feed.article_count === 1 ? "" : "s"} · {formatRelativeTime(feed.last_refreshed_at)}
            </span>
            {feed.status === "paused" ? <span className="bergag-badge">Paused</span> : null}
            {feed.health !== "healthy" ? (
              <span className="bergag-badge">{healthLabel(feed.health)}</span>
            ) : null}
          </span>
          <MoreHorizontal size={14} aria-hidden="true" />
        </button>
      </ContextMenu.Trigger>
      <ContextMenu.Portal>
        <ContextMenu.Content className="bergag-context-content">
          <ContextMenu.Item
            className="bergag-context-item"
            onSelect={() => refresh.mutate(feed.id)}
            disabled={refresh.isPending}
          >
            <RefreshCw size={13} aria-hidden="true" /> Refresh now
          </ContextMenu.Item>
          <ContextMenu.Item
            className="bergag-context-item"
            onSelect={() => {
              setSettingsFeed(feed);
              setSettingsMode("feed");
            }}
          >
            <MoreHorizontal size={13} aria-hidden="true" /> Feed settings
          </ContextMenu.Item>
          <ContextMenu.Separator className="bergag-context-separator" />
          <ContextMenu.Item className="bergag-context-item" onSelect={() => setPendingDelete(feed)}>
            <Trash2 size={13} aria-hidden="true" /> Delete feed
          </ContextMenu.Item>
        </ContextMenu.Content>
      </ContextMenu.Portal>
    </ContextMenu.Root>
  );

  const allFeeds = feeds.data ?? [];
  const grouped = (folders.data ?? []).map((folder) => ({
    folder,
    feeds: allFeeds.filter((feed) => feed.folder_id === folder.id),
  }));
  const unfoldered = allFeeds.filter((feed) => feed.folder_id === null);

  return (
    <div className="bergag-feed-view">
      <div className="bergag-feed-toolbar">
        <div className="bergag-inline-actions">
          <Button
            variant="primary"
            size="sm"
            onClick={() => setAddOpen(true)}
            icon={<Plus size={14} />}
          >
            Add feed
          </Button>
          <IconButton
            label="New folder"
            icon={<FolderPlus size={15} aria-hidden="true" />}
            onClick={() => {
              setSettingsFeed(null);
              setSettingsMode("folder");
            }}
          />
          <IconButton
            label="Refresh all due feeds"
            icon={<RefreshCw size={15} aria-hidden="true" />}
            onClick={() => refresh.mutate(null)}
            disabled={refresh.isPending}
          />
          <IconButton
            label="Import OPML"
            icon={<Upload size={15} aria-hidden="true" />}
            onClick={() => setImportOpen(true)}
          />
          <IconButton
            label="Export OPML"
            icon={<Download size={15} aria-hidden="true" />}
            onClick={() => setExportOpen(true)}
          />
        </div>
        <span className="bergag-feed-toolbar__count">
          {allFeeds.length} feed{allFeeds.length === 1 ? "" : "s"}
        </span>
      </div>

      {notice ? (
        <div className="bergag-feed-notice">
          <span>{notice}</span>
          <IconButton
            label="Dismiss notice"
            icon={<X size={13} aria-hidden="true" />}
            onClick={() => setNotice(null)}
          />
        </div>
      ) : null}
      {actionError ? (
        <div className="bergag-error" role="alert">
          {actionError}
        </div>
      ) : null}
      {feeds.error ? (
        <div className="bergag-error" role="alert">
          {errorMessage(feeds.error)}
        </div>
      ) : null}

      <div className="bergag-feed-list">
        {feeds.isLoading ? <div className="bergag-loading">Loading feeds…</div> : null}
        {!feeds.isLoading && allFeeds.length === 0 ? (
          <div className="bergag-empty-state">
            <div className="bergag-empty-state__title">No feeds yet</div>
            <p className="bergag-empty-state__text">
              Add a feed URL or import an OPML file to start building your local reading library.
            </p>
            <Button variant="secondary" onClick={() => setAddOpen(true)} icon={<Plus size={14} />}>
              Add your first feed
            </Button>
          </div>
        ) : null}

        {grouped.map(({ folder, feeds: folderFeeds }) => (
          <section key={folder.id} className="bergag-feed-group">
            <ContextMenu.Root>
              <ContextMenu.Trigger asChild>
                <div
                  className="bergag-feed-group__header"
                  aria-label={`${folder.name} folder actions`}
                >
                  <span>{folder.name}</span>
                  <span className="bergag-feed-group__count">{folderFeeds.length}</span>
                </div>
              </ContextMenu.Trigger>
              <ContextMenu.Portal>
                <ContextMenu.Content className="bergag-context-content">
                  <ContextMenu.Item
                    className="bergag-context-item"
                    onSelect={() => setPendingFolderDelete(folder)}
                  >
                    <Trash2 size={13} aria-hidden="true" /> Delete folder
                  </ContextMenu.Item>
                </ContextMenu.Content>
              </ContextMenu.Portal>
            </ContextMenu.Root>
            {folderFeeds.map(renderFeed)}
          </section>
        ))}

        {unfoldered.length > 0 ? (
          <section className="bergag-feed-group">
            <div className="bergag-feed-group__header">
              <span>All feeds</span>
              <span className="bergag-feed-group__count">{unfoldered.length}</span>
            </div>
            {unfoldered.map(renderFeed)}
          </section>
        ) : null}
      </div>

      <AddFeedDialog
        open={addOpen}
        onOpenChange={setAddOpen}
        folders={folders.data ?? []}
        onSubscribed={(result) => void handleSubscribed(result)}
      />
      {settingsMode ? (
        <FeedSettingsDialog
          key={`${settingsMode}-${settingsFeed?.id ?? ""}`}
          open
          onOpenChange={(open) => {
            if (!open) setSettingsMode(null);
          }}
          mode={settingsMode}
          feed={settingsFeed}
          folders={folders.data ?? []}
          onSaved={() => void invalidate()}
        />
      ) : null}
      {importOpen ? (
        <ImportOpmlDialog
          key="import-opml"
          open
          onOpenChange={setImportOpen}
          onImported={() => void invalidate()}
        />
      ) : null}
      {exportOpen ? <ExportOpmlDialog key="export-opml" open onOpenChange={setExportOpen} /> : null}

      <Dialog.Root
        open={pendingDelete !== null}
        onOpenChange={(open) => !open && setPendingDelete(null)}
      >
        <Dialog.Portal>
          <Dialog.Overlay className="bergag-dialog-overlay" />
          <Dialog.Content
            className="bergag-dialog"
            style={{ width: "min(440px, calc(100vw - 48px))", height: "auto" }}
          >
            <div className="bergag-dialog__body">
              <div className="bergag-dialog__header">
                <Dialog.Title className="bergag-title bergag-title--small">
                  Delete feed?
                </Dialog.Title>
                <Dialog.Close asChild>
                  <IconButton label="Cancel delete" icon={<X size={15} aria-hidden="true" />} />
                </Dialog.Close>
              </div>
              <div className="bergag-dialog__content">
                <p className="bergag-subtitle">
                  This removes “{pendingDelete?.title}” and its locally stored items from this
                  profile. This cannot be undone.
                </p>
                <div className="bergag-inline-actions" style={{ marginTop: 18 }}>
                  <Button
                    variant="danger"
                    onClick={() => pendingDelete && remove.mutate(pendingDelete.id)}
                    disabled={remove.isPending}
                    icon={<Trash2 size={14} />}
                  >
                    Delete feed
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

      <Dialog.Root
        open={pendingFolderDelete !== null}
        onOpenChange={(open) => !open && setPendingFolderDelete(null)}
      >
        <Dialog.Portal>
          <Dialog.Overlay className="bergag-dialog-overlay" />
          <Dialog.Content
            className="bergag-dialog"
            style={{ width: "min(440px, calc(100vw - 48px))", height: "auto" }}
          >
            <div className="bergag-dialog__body">
              <div className="bergag-dialog__header">
                <Dialog.Title className="bergag-title bergag-title--small">
                  Delete folder?
                </Dialog.Title>
                <Dialog.Close asChild>
                  <IconButton
                    label="Cancel folder delete"
                    icon={<X size={15} aria-hidden="true" />}
                  />
                </Dialog.Close>
              </div>
              <div className="bergag-dialog__content">
                <p className="bergag-subtitle">
                  The folder “{pendingFolderDelete?.name}” will be removed. Its feeds stay in your
                  library and move to All feeds.
                </p>
                <div className="bergag-inline-actions" style={{ marginTop: 18 }}>
                  <Button
                    variant="danger"
                    onClick={() =>
                      pendingFolderDelete && removeFolder.mutate(pendingFolderDelete.id)
                    }
                    disabled={removeFolder.isPending}
                    icon={<Folder size={14} />}
                  >
                    Delete folder
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
    </div>
  );
}
