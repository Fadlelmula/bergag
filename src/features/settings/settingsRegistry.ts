export type SettingsSectionId =
  | "general"
  | "appearance"
  | "layout"
  | "privacy"
  | "about"
  | "shortcuts"
  | "reading"
  | "discovery"
  | "notifications"
  | "tray"
  | "startup"
  | "updates"
  | "storage"
  | "offline"
  | "search"
  | "rules"
  | "ai"
  | "integrations"
  | "security"
  | "profiles"
  | "backups"
  | "advanced";

export interface SettingsSection {
  id: SettingsSectionId;
  label: string;
  description: string;
  milestone: number;
  available: boolean;
  keywords: string;
}

export const SETTINGS_SECTIONS: readonly SettingsSection[] = [
  {
    id: "general",
    label: "General",
    description: "Profiles and foundation preferences",
    milestone: 1,
    available: true,
    keywords: "profile switch rename delete",
  },
  {
    id: "appearance",
    label: "Appearance",
    description: "Theme, accent color, density, motion",
    milestone: 1,
    available: true,
    keywords: "dark light theme color accent typography density",
  },
  {
    id: "layout",
    label: "Layout",
    description: "Panels, sidebar, widths, reset",
    milestone: 1,
    available: true,
    keywords: "panels sidebar width resize reset",
  },
  {
    id: "privacy",
    label: "Privacy",
    description: "Local data inventory, network surfaces, and privacy status",
    milestone: 9,
    available: true,
    keywords: "telemetry local data storage privacy network dashboard",
  },
  {
    id: "about",
    label: "About & diagnostics",
    description: "Versions, integrity, recent logs",
    milestone: 1,
    available: true,
    keywords: "version diagnostics logs database integrity",
  },
  {
    id: "shortcuts",
    label: "Keyboard shortcuts",
    description: "Current shortcut map",
    milestone: 1,
    available: true,
    keywords: "keys keyboard shortcuts",
  },
  {
    id: "reading",
    label: "Reading",
    description: "Reader typography, list layout, and automatic read behavior",
    milestone: 3,
    available: true,
    keywords: "reader font mark read",
  },
  {
    id: "discovery",
    label: "Discovery",
    description: "Local directory and source trust",
    milestone: 4,
    available: true,
    keywords: "discover directory search sources",
  },
  {
    id: "notifications",
    label: "Notifications",
    description: "Real local notification preferences and quiet hours",
    milestone: 9,
    available: true,
    keywords: "notifications quiet hours",
  },
  {
    id: "tray",
    label: "System tray",
    description: "Tray controls and close behavior",
    milestone: 9,
    available: true,
    keywords: "tray close minimize background",
  },
  {
    id: "startup",
    label: "Startup",
    description: "Launch on sign-in behavior",
    milestone: 9,
    available: true,
    keywords: "startup launch sign in autostart",
  },
  {
    id: "updates",
    label: "Updates",
    description: "Signed update checks and release status",
    milestone: 9,
    available: true,
    keywords: "updates updater signature release",
  },
  {
    id: "storage",
    label: "Storage",
    description: "Disk usage and local cache totals",
    milestone: 6,
    available: true,
    keywords: "storage disk usage cache",
  },
  {
    id: "offline",
    label: "Offline",
    description: "Offline cache and image settings",
    milestone: 3,
    available: true,
    keywords: "offline cache images",
  },
  {
    id: "search",
    label: "Search",
    description: "Full-text search, saved searches, and story grouping",
    milestone: 5,
    available: true,
    keywords: "search fts semantic stories duplicates timeline",
  },
  {
    id: "rules",
    label: "Rules",
    description: "Local automation and smart feeds",
    milestone: 4,
    available: true,
    keywords: "rules automation",
  },
  {
    id: "ai",
    label: "AI",
    description: "Providers, privacy, and features",
    milestone: 7,
    available: true,
    keywords: "ai providers ollama openai",
  },
  {
    id: "integrations",
    label: "Integrations",
    description: "Public feed adapters and connected sources",
    milestone: 8,
    available: true,
    keywords: "integrations youtube reddit github podcast mastodon rss",
  },
  {
    id: "security",
    label: "Security",
    description: "Local security posture and credentials",
    milestone: 9,
    available: true,
    keywords: "security credentials",
  },
  {
    id: "profiles",
    label: "Profiles",
    description: "Local profile state and lock status",
    milestone: 9,
    available: true,
    keywords: "profiles lock export",
  },
  {
    id: "backups",
    label: "Backups",
    description: "Verified local backup creation and restore",
    milestone: 9,
    available: true,
    keywords: "backup restore checksum integrity retention",
  },
  {
    id: "advanced",
    label: "Advanced",
    description: "Diagnostics and developer options",
    milestone: 9,
    available: false,
    keywords: "advanced developer",
  },
];
