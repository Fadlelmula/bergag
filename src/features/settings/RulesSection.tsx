import { useQuery } from "@tanstack/react-query";
import { queryKeys } from "../../application/queryKeys";
import { desktop } from "../../infrastructure/tauri/commands";

export function RulesSection() {
  const rules = useQuery({
    queryKey: queryKeys.rules("settings"),
    queryFn: desktop.listRules,
  });

  const total = (rules.data ?? []).length;
  const enabled = (rules.data ?? []).filter((rule) => rule.enabled).length;
  const smartFeeds = (rules.data ?? []).filter((rule) => rule.is_smart_feed).length;

  return (
    <div>
      <p className="bergag-eyebrow">Rules</p>
      <h2 className="bergag-title bergag-title--small">Local automation</h2>
      <p className="bergag-subtitle">
        Rules match stored articles and apply state or label changes locally. Runs are always
        explicit; Bergag never changes your library silently in the background.
      </p>

      <div className="bergag-card bergag-card--padded" style={{ marginTop: 18 }}>
        <div className="bergag-switch-row">
          <div className="bergag-switch-row__text">
            <span className="bergag-switch-row__label">Rules</span>
            <span className="bergag-switch-row__hint">Total rules in this profile.</span>
          </div>
          <span className="bergag-badge">{rules.isLoading ? "…" : total}</span>
        </div>
        <div className="bergag-switch-row">
          <div className="bergag-switch-row__text">
            <span className="bergag-switch-row__label">Enabled</span>
            <span className="bergag-switch-row__hint">
              Included when you choose Run enabled rules.
            </span>
          </div>
          <span className="bergag-badge">{rules.isLoading ? "…" : enabled}</span>
        </div>
        <div className="bergag-switch-row">
          <div className="bergag-switch-row__text">
            <span className="bergag-switch-row__label">Smart feeds</span>
            <span className="bergag-switch-row__hint">
              Read-only views built from a rule&apos;s conditions.
            </span>
          </div>
          <span className="bergag-badge">{rules.isLoading ? "…" : smartFeeds}</span>
        </div>
        <div className="bergag-switch-row">
          <div className="bergag-switch-row__text">
            <span className="bergag-switch-row__label">Background execution</span>
            <span className="bergag-switch-row__hint">
              Not enabled. Rules only run when you start them.
            </span>
          </div>
          <span className="bergag-badge bergag-badge--accent">Off</span>
        </div>
      </div>

      <div className="bergag-note" style={{ marginTop: 16 }}>
        Build and run rules in the Discover view. Run history is stored locally and can be cleared
        by deleting the rule.
      </div>
    </div>
  );
}
