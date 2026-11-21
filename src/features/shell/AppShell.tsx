import { useLayout } from "../../app/LayoutProvider";
import { useProfile } from "../../app/ProfileProvider";
import { isArticleView } from "../../application/articleService";
import { invalidateProfileData } from "../../application/profileInvalidation";
import { queryKeys } from "../../application/queryKeys";
import { Button } from "../../components/ui/Button";
import { ResizeHandle } from "../../components/ui/ResizeHandle";
import { DEFAULT_LAYOUT, type BergagLayout, type PanelId } from "../../domain/layout";
import type { NavItemId } from "../../domain/navigation";
import { desktop } from "../../infrastructure/tauri/commands";
import { SettingsDialog } from "../settings/SettingsDialog";
import type { SettingsSectionId } from "../settings/settingsRegistry";
import { ShortcutsDialog } from "../shortcuts/ShortcutsDialog";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { listen } from "@tauri-apps/api/event";
import * as Tooltip from "@radix-ui/react-tooltip";
import { useCallback, useEffect, useRef, useState } from "react";
import { AIView } from "../ai/AIView";
import { ArticleActivityView } from "../articles/ArticleActivityView";
import { ArticleList } from "../articles/ArticleList";
import { ArticleReader } from "../articles/ArticleReader";
import { BriefingsView } from "../analytics/BriefingsView";
import { StatisticsView } from "../analytics/StatisticsView";
import { FeedDetail } from "../feeds/FeedDetail";
import { FeedsView } from "../feeds/FeedsView";
import { DiscoveryView } from "../discovery/DiscoveryView";
import { TopicsView } from "../labels/TopicsView";
import { SearchView } from "../search/SearchView";
import { isTauriRuntime } from "../../infrastructure/tauri/client";
import { FoundationView } from "./FoundationView";
import { ReadingPanePlaceholder } from "./ReadingPanePlaceholder";
import { Sidebar } from "./Sidebar";

function isTypingTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  return (
    target.isContentEditable ||
    target.tagName === "INPUT" ||
    target.tagName === "TEXTAREA" ||
    target.tagName === "SELECT"
  );
}

function clamp(value: number, minimum: number, maximum: number): number {
  return Math.min(maximum, Math.max(minimum, Math.round(value)));
}

type ResizeTarget = "sidebar" | "articleList";

export function AppShell() {
  const { activeProfile, profiles, isSwitching, switchProfile, error: profileError } = useProfile();
  const { layout, updateLayout, error: layoutError } = useLayout();
  const queryClient = useQueryClient();
  const diagnostics = useQuery({
    queryKey: queryKeys.diagnostics,
    queryFn: desktop.getDiagnostics,
  });

  const [activeView, setActiveView] = useState<NavItemId>("home");
  const [selectedArticleId, setSelectedArticleId] = useState<string | null>(null);
  const [focusMode, setFocusMode] = useState(false);
  const [selection, setSelection] = useState<{ feedId: string; profileId: string } | null>(null);
  const selectFeed = useCallback(
    (feedId: string) => setSelection({ feedId, profileId: activeProfile?.id ?? "" }),
    [activeProfile?.id],
  );
  const selectedFeedId =
    selection && selection.profileId === activeProfile?.id ? selection.feedId : null;
  const feedList = useQuery({
    queryKey: queryKeys.feeds(activeProfile?.id ?? ""),
    queryFn: desktop.listFeeds,
    enabled: activeView === "allFeeds" && Boolean(activeProfile),
  });
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [settingsSection, setSettingsSection] = useState<SettingsSectionId>("general");
  const [shortcutsOpen, setShortcutsOpen] = useState(false);
  const pendingG = useRef(false);

  useEffect(() => {
    if (!isTauriRuntime()) return;
    let unlisten: (() => void) | undefined;
    void listen("feeds:refreshed", () => {
      void invalidateProfileData(queryClient, [activeProfile?.id]);
      void queryClient.invalidateQueries({
        queryKey: queryKeys.feedFolders(activeProfile?.id ?? ""),
      });
    })
      .then((callback) => {
        unlisten = callback;
      })
      .catch(() => undefined);
    return () => unlisten?.();
  }, [queryClient, activeProfile?.id]);

  const openSettings = useCallback((section: SettingsSectionId = "general") => {
    setSettingsSection(section);
    setSettingsOpen(true);
  }, []);

  const navigate = useCallback(
    (id: NavItemId) => {
      if (id === "settings") {
        openSettings("general");
        return;
      }
      setSelectedArticleId(null);
      setFocusMode(false);
      setActiveView(id);
    },
    [openSettings],
  );

  const switchProfileAndReset = useCallback(
    (profileId: string) => {
      setSelectedArticleId(null);
      setFocusMode(false);
      void switchProfile(profileId);
    },
    [switchProfile],
  );

  const openArticleFromActivity = useCallback((id: string) => {
    setSelectedArticleId(id);
    setFocusMode(false);
    setActiveView("unread");
  }, []);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (isTypingTarget(event.target) || event.ctrlKey || event.metaKey || event.altKey) return;

      if (event.key === "Escape") {
        if (settingsOpen || shortcutsOpen) {
          setSettingsOpen(false);
          setShortcutsOpen(false);
        } else if (focusMode && isArticleView(activeView)) {
          setFocusMode(false);
        }
        pendingG.current = false;
        return;
      }
      if (settingsOpen || shortcutsOpen) return;
      if (event.key === "?") {
        event.preventDefault();
        setShortcutsOpen(true);
        return;
      }
      if (event.key === "/") {
        event.preventDefault();
        openSettings("general");
        return;
      }
      if (pendingG.current) {
        pendingG.current = false;
        if (event.key.toLowerCase() === "h") {
          event.preventDefault();
          navigate("home");
          return;
        }
        if (event.key.toLowerCase() === "s") {
          event.preventDefault();
          openSettings("general");
          return;
        }
      }
      if (event.key.toLowerCase() === "f" && isArticleView(activeView)) {
        event.preventDefault();
        setFocusMode((current) => !current);
        return;
      }
      if (event.key.toLowerCase() === "g") pendingG.current = true;
    };

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [activeView, focusMode, navigate, openSettings, settingsOpen, shortcutsOpen]);

  const isPanelVisible = (id: PanelId) => {
    if (id === "sidebar") return layout.sidebar.visible;
    if (id === "articleList") return layout.panels.articleList.visible;
    return layout.panels.readingPane.visible;
  };

  const updateSidebar = (
    updater: (sidebar: BergagLayout["sidebar"]) => BergagLayout["sidebar"],
  ) => {
    updateLayout((current) => ({ ...current, sidebar: updater(current.sidebar) }));
  };

  const moveItem = (id: NavItemId, direction: -1 | 1) => {
    const visible = layout.sidebar.order.filter((item) => !layout.sidebar.hidden.includes(item));
    const index = visible.indexOf(id);
    const target = visible[index + direction];
    if (!target) return;
    updateSidebar((sidebar) => {
      const order = [...sidebar.order];
      const from = order.indexOf(id);
      const to = order.indexOf(target);
      if (from < 0 || to < 0) return sidebar;
      const moved = order[from];
      order[from] = order[to] as NavItemId;
      order[to] = moved as NavItemId;
      return { ...sidebar, order };
    });
  };

  const resizePanel = (target: ResizeTarget, delta: number, persist: boolean) => {
    if (target === "sidebar") {
      updateLayout(
        (current) => ({
          ...current,
          sidebar: {
            ...current.sidebar,
            width: clamp(current.sidebar.width + delta, 200, 420),
          },
        }),
        persist,
      );
    } else {
      updateLayout(
        (current) => ({
          ...current,
          panels: {
            ...current.panels,
            articleList: {
              ...current.panels.articleList,
              width: clamp(current.panels.articleList.width + delta, 280, 720),
            },
          },
        }),
        persist,
      );
    }
  };

  const articleView = isArticleView(activeView);
  const fullView =
    activeView === "discover" ||
    activeView === "topics" ||
    activeView === "search" ||
    activeView === "briefings" ||
    activeView === "statistics" ||
    activeView === "ai";
  const visiblePanels =
    focusMode && articleView
      ? (["readingPane"] as PanelId[])
      : fullView
        ? layout.order.filter((panelId) =>
            panelId === "sidebar" ? layout.sidebar.visible : panelId === "readingPane",
          )
        : layout.order.filter(isPanelVisible);
  const segments: Array<
    { kind: "panel"; id: PanelId } | { kind: "handle"; target: ResizeTarget | null }
  > = [];
  visiblePanels.forEach((panelId, index) => {
    segments.push({ kind: "panel", id: panelId });
    if (index < visiblePanels.length - 1) {
      const next = visiblePanels[index + 1];
      const target: ResizeTarget | null =
        panelId === "sidebar"
          ? "sidebar"
          : panelId === "articleList"
            ? "articleList"
            : next === "sidebar"
              ? "sidebar"
              : next === "articleList"
                ? "articleList"
                : null;
      segments.push({ kind: "handle", target });
    }
  });

  const columns = segments
    .map((segment) => {
      if (segment.kind === "handle") return "6px";
      if (segment.id === "sidebar") {
        return `${layout.sidebar.collapsed ? 56 : layout.sidebar.width}px`;
      }
      if (segment.id === "articleList") return `${layout.panels.articleList.width}px`;
      return "minmax(0, 1fr)";
    })
    .join(" ");

  const panelStyle = { gridTemplateColumns: columns || "minmax(0, 1fr)" };

  const renderPanel = (id: PanelId) => {
    if (id === "sidebar") {
      return (
        <div className="bergag-panel bergag-panel--sidebar" key="sidebar">
          <Sidebar
            collapsed={layout.sidebar.collapsed}
            width={layout.sidebar.width}
            order={layout.sidebar.order}
            hidden={layout.sidebar.hidden}
            activeView={activeView}
            profiles={profiles}
            activeProfile={activeProfile}
            isSwitching={isSwitching}
            onNavigate={navigate}
            onToggleCollapse={() =>
              updateSidebar((sidebar) => ({ ...sidebar, collapsed: !sidebar.collapsed }))
            }
            onHide={(navId) =>
              updateSidebar((sidebar) => ({
                ...sidebar,
                hidden: sidebar.hidden.includes(navId)
                  ? sidebar.hidden
                  : [...sidebar.hidden, navId],
              }))
            }
            onShow={(navId) =>
              updateSidebar((sidebar) => ({
                ...sidebar,
                hidden: sidebar.hidden.filter((item) => item !== navId),
              }))
            }
            onMove={moveItem}
            onResetOrder={() =>
              updateSidebar((sidebar) => ({
                ...sidebar,
                order: [...DEFAULT_LAYOUT.sidebar.order],
                hidden: [],
              }))
            }
            onSwitchProfile={switchProfileAndReset}
          />
        </div>
      );
    }
    if (id === "articleList") {
      return (
        <div className="bergag-panel bergag-panel--list" key="articleList">
          {activeView === "home" ? (
            <FoundationView
              activeProfile={activeProfile}
              profileCount={profiles.length}
              diagnostics={diagnostics.data}
              diagnosticsLoading={diagnostics.isLoading}
              diagnosticsError={
                diagnostics.error
                  ? diagnostics.error instanceof Error
                    ? diagnostics.error.message
                    : String(diagnostics.error)
                  : null
              }
              onOpenSettings={() => openSettings("privacy")}
            />
          ) : activeView === "allFeeds" ? (
            <FeedsView
              profileId={activeProfile?.id ?? ""}
              selectedFeedId={selectedFeedId}
              onSelectFeed={selectFeed}
            />
          ) : (
            <ArticleList
              key={`${activeProfile?.id ?? ""}:${activeView}`}
              profileId={activeProfile?.id ?? ""}
              activeView={activeView}
              selectedArticleId={selectedArticleId}
              onSelectArticle={setSelectedArticleId}
              onOpenSettings={() => openSettings("reading")}
            />
          )}
        </div>
      );
    }
    return (
      <div className="bergag-panel bergag-panel--reader" key="readingPane">
        {activeView === "allFeeds" ? (
          <FeedDetail
            profileId={activeProfile?.id ?? ""}
            feed={(feedList.data ?? []).find((feed) => feed.id === selectedFeedId)}
          />
        ) : activeView === "discover" ? (
          <DiscoveryView
            profileId={activeProfile?.id ?? ""}
            onOpenArticle={openArticleFromActivity}
          />
        ) : activeView === "topics" ? (
          <TopicsView profileId={activeProfile?.id ?? ""} onOpenArticle={openArticleFromActivity} />
        ) : activeView === "search" ? (
          <SearchView profileId={activeProfile?.id ?? ""} onOpenArticle={openArticleFromActivity} />
        ) : activeView === "briefings" ? (
          <BriefingsView
            profileId={activeProfile?.id ?? ""}
            onOpenArticle={openArticleFromActivity}
          />
        ) : activeView === "statistics" ? (
          <StatisticsView
            profileId={activeProfile?.id ?? ""}
            onOpenArticle={openArticleFromActivity}
          />
        ) : activeView === "ai" ? (
          <AIView onOpenSettings={() => openSettings("ai")} />
        ) : activeView === "history" || activeView === "highlights" ? (
          <ArticleActivityView
            kind={activeView}
            profileId={activeProfile?.id ?? ""}
            onSelectArticle={openArticleFromActivity}
          />
        ) : articleView ? (
          <ArticleReader
            key={`${activeProfile?.id ?? ""}:${activeView}:${selectedArticleId ?? "none"}`}
            profileId={activeProfile?.id ?? ""}
            articleId={selectedArticleId}
            focusMode={focusMode}
            onToggleFocusMode={() => setFocusMode((current) => !current)}
            onOpenSettings={() => openSettings("reading")}
            onOpenArticle={openArticleFromActivity}
            isShortcutBlocked={() => pendingG.current}
          />
        ) : (
          <ReadingPanePlaceholder layout={layout} />
        )}
      </div>
    );
  };

  const integrity = diagnostics.data?.database.integrity_check ?? "checking";
  const shellError = profileError ?? layoutError;

  return (
    <Tooltip.Provider delayDuration={250}>
      <div className="bergag-app">
        <a className="bergag-skip-link" href="#bergag-workspace">
          Skip to workspace
        </a>
        <main
          id="bergag-workspace"
          className="bergag-workspace"
          style={panelStyle}
          aria-label="Bergag workspace"
          tabIndex={-1}
        >
          {segments.map((segment, index) => {
            if (segment.kind === "panel") return renderPanel(segment.id);
            const value =
              segment.target === "sidebar"
                ? layout.sidebar.width
                : segment.target === "articleList"
                  ? layout.panels.articleList.width
                  : 0;
            const bounds =
              segment.target === "sidebar"
                ? { min: 200, max: 420 }
                : segment.target === "articleList"
                  ? { min: 280, max: 720 }
                  : { min: 0, max: 100 };
            return (
              <ResizeHandle
                key={`handle-${index}`}
                label={segment.target ? `Resize ${segment.target} panel` : "Panel divider"}
                value={value}
                min={bounds.min}
                max={bounds.max}
                interactive={Boolean(segment.target)}
                onDelta={(delta) => {
                  if (segment.target) resizePanel(segment.target, delta, false);
                }}
                onCommit={() => {
                  if (segment.target) resizePanel(segment.target, 0, true);
                }}
              />
            );
          })}
        </main>

        <footer className="bergag-status-bar">
          <span className="bergag-status-item">
            <span className="bergag-status-dot" aria-hidden="true" />
            Database {integrity}
          </span>
          <span className="bergag-status-item">Profile: {activeProfile?.name ?? "—"}</span>
          <span className="bergag-status-item">Telemetry off</span>
          <span className="bergag-status-bar__spacer" />
          {shellError ? (
            <span className="bergag-status-item" role="alert">
              {shellError}
            </span>
          ) : null}
          <Button variant="ghost" size="sm" onClick={() => setShortcutsOpen(true)}>
            ? Shortcuts
          </Button>
        </footer>
      </div>

      <SettingsDialog
        key={settingsSection}
        open={settingsOpen}
        onOpenChange={setSettingsOpen}
        initialSection={settingsSection}
      />
      <ShortcutsDialog open={shortcutsOpen} onOpenChange={setShortcutsOpen} />
    </Tooltip.Provider>
  );
}
