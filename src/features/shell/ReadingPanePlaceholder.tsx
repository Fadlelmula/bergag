import { SpiderLilyMark } from "../../components/branding/SpiderLilyMark";
import type { BergagLayout } from "../../domain/layout";

export function ReadingPanePlaceholder({ layout }: { layout: BergagLayout }) {
  const mode = layout.panels.readingPane.visible
    ? layout.sidebar.collapsed
      ? "compact shell"
      : "side-by-side shell"
    : "hidden";

  return (
    <div className="bergag-empty-state" data-testid="reading-pane-placeholder">
      <div className="bergag-empty-state__mark">
        <SpiderLilyMark size={52} />
      </div>
      <h2 className="bergag-empty-state__title">Reader is ready</h2>
      <p className="bergag-empty-state__text">
        Article reading, extraction, notes, highlights, and focus mode are implemented in Milestone
        3. No sample articles are shown because Bergag never fabricates library content.
      </p>
      <p className="bergag-stat__detail" style={{ marginTop: 14 }}>
        Reading pane mode: {mode} · density: {layout.density}
      </p>
    </div>
  );
}
