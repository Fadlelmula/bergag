import * as ContextMenu from "@radix-ui/react-context-menu";
import {
  ArrowDown,
  ArrowUp,
  ChevronRight,
  EyeOff,
  PanelLeftClose,
  PanelLeftOpen,
  RotateCcw,
} from "lucide-react";
import { SpiderLilyMark } from "../../components/branding/SpiderLilyMark";
import { Hint } from "../../components/ui/Hint";
import { IconButton } from "../../components/ui/IconButton";
import type { Profile } from "../../domain/models";
import { getNavItem, type NavItemId } from "../../domain/navigation";
import { NAV_ICONS } from "./navIcons";

interface SidebarProps {
  collapsed: boolean;
  width: number;
  order: NavItemId[];
  hidden: NavItemId[];
  activeView: NavItemId;
  profiles: Profile[];
  activeProfile: Profile | undefined;
  isSwitching: boolean;
  onNavigate: (id: NavItemId) => void;
  onToggleCollapse: () => void;
  onHide: (id: NavItemId) => void;
  onShow: (id: NavItemId) => void;
  onMove: (id: NavItemId, direction: -1 | 1) => void;
  onResetOrder: () => void;
  onSwitchProfile: (id: string) => void;
}

const SECTION_LABELS: Record<string, string> = {
  workspace: "Workspace",
  library: "Library",
  intelligence: "Intelligence",
  system: "System",
};

export function Sidebar({
  collapsed,
  width,
  order,
  hidden,
  activeView,
  profiles,
  activeProfile,
  isSwitching,
  onNavigate,
  onToggleCollapse,
  onHide,
  onShow,
  onMove,
  onResetOrder,
  onSwitchProfile,
}: SidebarProps) {
  const visible = order.filter((id) => !hidden.includes(id));
  const sections = ["workspace", "library", "intelligence", "system"];

  const navButton = (id: NavItemId) => {
    const definition = getNavItem(id);
    const Icon = NAV_ICONS[id];
    const isActive = activeView === id;
    const reason = definition.available
      ? definition.description
      : `Planned for Milestone ${definition.milestone}: ${definition.description}`;
    const button = (
      <button
        type="button"
        data-testid={`nav-${id}`}
        className={`bergag-nav-item ${isActive ? "bergag-nav-item--active" : ""}`}
        aria-current={isActive ? "page" : undefined}
        aria-disabled={!definition.available}
        disabled={!definition.available}
        onClick={() => definition.available && onNavigate(id)}
      >
        <Icon size={16} strokeWidth={1.8} aria-hidden="true" />
        {!collapsed ? <span className="bergag-nav-item__label">{definition.label}</span> : null}
        {!collapsed && !definition.available ? (
          <span className="bergag-nav-item__planned">M{definition.milestone}</span>
        ) : null}
      </button>
    );

    return (
      <ContextMenu.Root key={id}>
        <ContextMenu.Trigger asChild>
          <Hint content={reason} side="right">
            {button}
          </Hint>
        </ContextMenu.Trigger>
        <ContextMenu.Portal>
          <ContextMenu.Content className="bergag-context-content">
            {definition.available ? (
              <ContextMenu.Item className="bergag-context-item" onSelect={() => onNavigate(id)}>
                Open
              </ContextMenu.Item>
            ) : (
              <ContextMenu.Item className="bergag-context-item" disabled>
                Planned for Milestone {definition.milestone}
              </ContextMenu.Item>
            )}
            <ContextMenu.Separator className="bergag-context-separator" />
            <ContextMenu.Item
              className="bergag-context-item"
              onSelect={() => onMove(id, -1)}
              disabled={visible.indexOf(id) <= 0}
            >
              <ArrowUp size={13} aria-hidden="true" /> Move up
            </ContextMenu.Item>
            <ContextMenu.Item
              className="bergag-context-item"
              onSelect={() => onMove(id, 1)}
              disabled={visible.indexOf(id) === visible.length - 1}
            >
              <ArrowDown size={13} aria-hidden="true" /> Move down
            </ContextMenu.Item>
            <ContextMenu.Item className="bergag-context-item" onSelect={() => onHide(id)}>
              <EyeOff size={13} aria-hidden="true" /> Hide from sidebar
            </ContextMenu.Item>
            <ContextMenu.Separator className="bergag-context-separator" />
            <ContextMenu.Item className="bergag-context-item" onSelect={onResetOrder}>
              <RotateCcw size={13} aria-hidden="true" /> Reset sidebar order
            </ContextMenu.Item>
          </ContextMenu.Content>
        </ContextMenu.Portal>
      </ContextMenu.Root>
    );
  };

  return (
    <aside
      className="bergag-sidebar"
      style={{ width: collapsed ? 56 : width }}
      aria-label="Bergag sidebar"
    >
      <div className="bergag-sidebar__header">
        <span className="bergag-sidebar__mark">
          <SpiderLilyMark size={26} />
        </span>
        {!collapsed ? (
          <div>
            <div className="bergag-sidebar__title">BERGAG</div>
            <div className="bergag-sidebar__subtitle">Local intelligence</div>
          </div>
        ) : null}
        <div style={{ marginLeft: "auto" }}>
          <Hint content={collapsed ? "Expand sidebar" : "Collapse sidebar"} side="right">
            <IconButton
              label={collapsed ? "Expand sidebar" : "Collapse sidebar"}
              icon={
                collapsed ? (
                  <PanelLeftOpen size={15} aria-hidden="true" />
                ) : (
                  <PanelLeftClose size={15} aria-hidden="true" />
                )
              }
              onClick={onToggleCollapse}
            />
          </Hint>
        </div>
      </div>

      <div className="bergag-sidebar__profile">
        {!collapsed ? <span className="bergag-sidebar__profile-label">Profile</span> : null}
        <select
          className="bergag-select"
          aria-label="Active profile"
          value={activeProfile?.id ?? ""}
          disabled={isSwitching || profiles.length === 0}
          onChange={(event) => onSwitchProfile(event.target.value)}
        >
          {profiles.length === 0 ? <option value="">Loading…</option> : null}
          {profiles.map((profile) => (
            <option key={profile.id} value={profile.id}>
              {profile.name}
            </option>
          ))}
        </select>
      </div>

      <nav className="bergag-sidebar__nav" aria-label="Primary">
        {sections.map((section) => {
          const items = visible.filter((id) => getNavItem(id).section === section);
          if (items.length === 0) return null;
          return (
            <div key={section}>
              {!collapsed ? (
                <div className="bergag-nav-section">{SECTION_LABELS[section]}</div>
              ) : null}
              {items.map(navButton)}
            </div>
          );
        })}

        {hidden.length > 0 && !collapsed ? (
          <details style={{ marginTop: 16 }}>
            <summary className="bergag-nav-section" style={{ cursor: "pointer" }}>
              Hidden items ({hidden.length})
            </summary>
            {hidden.map((id) => (
              <button
                key={id}
                type="button"
                className="bergag-nav-item"
                onClick={() => onShow(id)}
                title={`Show ${getNavItem(id).label}`}
              >
                <ChevronRight size={14} aria-hidden="true" />
                <span className="bergag-nav-item__label">Show {getNavItem(id).label}</span>
              </button>
            ))}
          </details>
        ) : null}
      </nav>
    </aside>
  );
}
