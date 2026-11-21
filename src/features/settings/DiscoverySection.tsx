import { useQuery } from "@tanstack/react-query";
import { queryKeys } from "../../application/queryKeys";
import { desktop } from "../../infrastructure/tauri/commands";

export function DiscoverySection() {
  const sources = useQuery({
    queryKey: queryKeys.discoverySources("settings"),
    queryFn: desktop.listDiscoverySources,
  });

  return (
    <div>
      <p className="bergag-eyebrow">Discovery</p>
      <h2 className="bergag-title bergag-title--small">Local directory</h2>
      <p className="bergag-subtitle">
        Discovery uses only the sources you save and fetch directly. Bergag does not contact a
        third-party search provider or directory service.
      </p>

      <div className="bergag-card bergag-card--padded" style={{ marginTop: 18 }}>
        <div className="bergag-switch-row">
          <div className="bergag-switch-row__text">
            <span className="bergag-switch-row__label">Saved sources</span>
            <span className="bergag-switch-row__hint">
              Sources stored in this profile&apos;s local directory.
            </span>
          </div>
          <span className="bergag-badge">
            {sources.isLoading ? "…" : (sources.data ?? []).length}
          </span>
        </div>
        <div className="bergag-switch-row">
          <div className="bergag-switch-row__text">
            <span className="bergag-switch-row__label">Third-party discovery service</span>
            <span className="bergag-switch-row__hint">Not used. There is no remote directory.</span>
          </div>
          <span className="bergag-badge bergag-badge--accent">Off</span>
        </div>
        <div className="bergag-switch-row">
          <div className="bergag-switch-row__text">
            <span className="bergag-switch-row__label">Public URL validation</span>
            <span className="bergag-switch-row__hint">
              Private, loopback, link-local, and non-HTTP addresses are rejected before any fetch.
            </span>
          </div>
          <span className="bergag-badge">Enforced</span>
        </div>
      </div>

      <div className="bergag-note" style={{ marginTop: 16 }}>
        Add, preview, and subscribe from the Discover view. Nothing is fetched until you choose
        Preview or Subscribe.
      </div>
    </div>
  );
}
