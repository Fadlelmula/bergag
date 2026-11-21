import { Compass, ListFilter } from "lucide-react";
import { useState } from "react";
import { DirectoryPanel } from "./DirectoryPanel";
import { RulesPanel } from "./RulesPanel";

type DiscoveryTab = "directory" | "rules";

interface DiscoveryViewProps {
  profileId: string;
  onOpenArticle: (id: string) => void;
}

export function DiscoveryView({ profileId, onOpenArticle }: DiscoveryViewProps) {
  const [tab, setTab] = useState<DiscoveryTab>("directory");

  return (
    <div className="bergag-full-view">
      <div className="bergag-full-view__header">
        <div>
          <h2 className="bergag-title">Discover</h2>
          <p className="bergag-subtitle">
            A private local directory and local automation. Nothing is fetched or changed until you
            choose an action.
          </p>
        </div>
        <div className="bergag-segmented" role="tablist" aria-label="Discovery sections">
          <button
            type="button"
            role="tab"
            aria-selected={tab === "directory"}
            className={`bergag-segmented__item ${tab === "directory" ? "bergag-segmented__item--active" : ""}`}
            onClick={() => setTab("directory")}
          >
            <Compass size={14} aria-hidden="true" /> Directory
          </button>
          <button
            type="button"
            role="tab"
            aria-selected={tab === "rules"}
            className={`bergag-segmented__item ${tab === "rules" ? "bergag-segmented__item--active" : ""}`}
            onClick={() => setTab("rules")}
          >
            <ListFilter size={14} aria-hidden="true" /> Rules & smart feeds
          </button>
        </div>
      </div>
      <div className="bergag-full-view__body">
        {tab === "directory" ? (
          <DirectoryPanel
            profileId={profileId}
            onSubscribed={() => {
              // Feed queries are invalidated by the panel; the shell listens for refresh events.
            }}
          />
        ) : (
          <RulesPanel profileId={profileId} onOpenArticle={onOpenArticle} />
        )}
      </div>
    </div>
  );
}
