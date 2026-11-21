import * as Dialog from "@radix-ui/react-dialog";
import { X } from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";
import { IconButton } from "../../components/ui/IconButton";
import { AboutSection } from "./AboutSection";
import { AISection } from "./AISection";
import { AppearanceSection } from "./AppearanceSection";
import { BackupsSection } from "./BackupsSection";
import { DiscoverySection } from "./DiscoverySection";
import { GeneralSection } from "./GeneralSection";
import { IntegrationsSection } from "./IntegrationsSection";
import { LayoutSection } from "./LayoutSection";
import { NotificationsSection } from "./NotificationsSection";
import { ProfilesSection } from "./ProfilesSection";
import { SecuritySection } from "./SecuritySection";
import { OfflineSection } from "./OfflineSection";
import { PrivacySection } from "./PrivacySection";
import { ReadingSection } from "./ReadingSection";
import { RulesSection } from "./RulesSection";
import { SearchSection } from "./SearchSection";
import { StorageSection } from "./StorageSection";
import { StartupSection } from "./StartupSection";
import { TraySection } from "./TraySection";
import { UpdatesSection } from "./UpdatesSection";
import { SETTINGS_SECTIONS, type SettingsSectionId } from "./settingsRegistry";
import { ShortcutsSection } from "./ShortcutsSection";

interface SettingsDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  initialSection?: SettingsSectionId;
}

export function renderSection(id: SettingsSectionId) {
  switch (id) {
    case "general":
      return <GeneralSection />;
    case "appearance":
      return <AppearanceSection />;
    case "layout":
      return <LayoutSection />;
    case "privacy":
      return <PrivacySection />;
    case "about":
      return <AboutSection />;
    case "shortcuts":
      return <ShortcutsSection />;
    case "reading":
      return <ReadingSection />;
    case "offline":
      return <OfflineSection />;
    case "discovery":
      return <DiscoverySection />;
    case "rules":
      return <RulesSection />;
    case "search":
      return <SearchSection />;
    case "storage":
      return <StorageSection />;
    case "ai":
      return <AISection />;
    case "integrations":
      return <IntegrationsSection />;
    case "backups":
      return <BackupsSection />;
    case "notifications":
      return <NotificationsSection />;
    case "tray":
      return <TraySection />;
    case "startup":
      return <StartupSection />;
    case "updates":
      return <UpdatesSection />;
    case "security":
      return <SecuritySection />;
    case "profiles":
      return <ProfilesSection />;
    default:
      return null;
  }
}

export function SettingsDialog({
  open,
  onOpenChange,
  initialSection = "general",
}: SettingsDialogProps) {
  const [selectedOverride, setSelectedOverride] = useState<SettingsSectionId | null>(null);
  const selected = selectedOverride ?? initialSection;
  const [query, setQuery] = useState("");
  const searchRef = useRef<HTMLInputElement>(null);
  const titleRef = useRef<HTMLHeadingElement>(null);
  const previousOverride = useRef<SettingsSectionId | null>(selectedOverride);

  useEffect(() => {
    if (open) searchRef.current?.focus();
  }, [open]);

  useEffect(() => {
    if (open && selectedOverride && selectedOverride !== previousOverride.current) {
      titleRef.current?.focus();
    }
    previousOverride.current = selectedOverride;
  }, [open, selectedOverride]);

  const filtered = useMemo(() => {
    const normalized = query.trim().toLowerCase();
    if (!normalized) return SETTINGS_SECTIONS;
    return SETTINGS_SECTIONS.filter((section) =>
      `${section.label} ${section.description} ${section.keywords}`
        .toLowerCase()
        .includes(normalized),
    );
  }, [query]);

  const active =
    SETTINGS_SECTIONS.find((section) => section.id === selected) ?? SETTINGS_SECTIONS[0];
  const available = filtered.filter((section) => section.available);
  const planned = filtered.filter((section) => !section.available);
  if (!active) return null;

  const navItem = (section: (typeof SETTINGS_SECTIONS)[number]) => (
    <li key={section.id}>
      <button
        type="button"
        className={`bergag-dialog__nav-item ${selected === section.id ? "bergag-dialog__nav-item--active" : ""}`}
        aria-disabled={!section.available}
        aria-current={selected === section.id ? "true" : undefined}
        aria-label={
          section.available
            ? section.label
            : `${section.label}, planned for milestone ${section.milestone}`
        }
        onClick={() => {
          if (section.available) setSelectedOverride(section.id);
        }}
        title={
          section.available ? section.description : `Planned for Milestone ${section.milestone}`
        }
      >
        <span>{section.label}</span>
        {!section.available ? <span className="bergag-badge">M{section.milestone}</span> : null}
      </button>
    </li>
  );

  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay className="bergag-dialog-overlay" />
        <Dialog.Content className="bergag-dialog" aria-describedby="settings-description">
          <div className="bergag-dialog__sidebar">
            <div className="bergag-dialog__search">
              <input
                ref={searchRef}
                className="bergag-input"
                type="search"
                aria-label="Search settings"
                placeholder="Search settings…"
                value={query}
                onChange={(event) => setQuery(event.target.value)}
              />
            </div>
            <nav aria-label="Settings sections">
              {available.length > 0 ? (
                <>
                  <div className="bergag-dialog__section-label">Available now</div>
                  <ul>{available.map(navItem)}</ul>
                </>
              ) : null}
              {planned.length > 0 ? (
                <>
                  <div className="bergag-dialog__section-label">Planned</div>
                  <ul>{planned.map(navItem)}</ul>
                </>
              ) : null}
            </nav>
            {filtered.length === 0 ? (
              <p className="bergag-field__hint" role="status">
                No settings match that search.
              </p>
            ) : null}
          </div>

          <div className="bergag-dialog__body">
            <div className="bergag-dialog__header">
              <div>
                <Dialog.Title asChild>
                  <h1 ref={titleRef} tabIndex={-1} className="bergag-title bergag-title--small">
                    {active.label}
                  </h1>
                </Dialog.Title>
                <Dialog.Description id="settings-description" className="bergag-subtitle">
                  {active.description}
                </Dialog.Description>
              </div>
              <Dialog.Close asChild>
                <IconButton label="Close settings" icon={<X size={15} aria-hidden="true" />} />
              </Dialog.Close>
            </div>
            <div className="bergag-dialog__content">{renderSection(active.id)}</div>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
