import { SHORTCUTS } from "../shortcuts/ShortcutsDialog";

export function ShortcutsSection() {
  return (
    <div>
      <p className="bergag-eyebrow">Keyboard</p>
      <h2 className="bergag-title bergag-title--small">Keyboard map</h2>
      <p className="bergag-subtitle">
        Shortcuts are context-aware and ignored while typing. Remapping is not implemented yet.
      </p>
      <div style={{ marginTop: 18 }}>
        {SHORTCUTS.map((shortcut) => (
          <div className="bergag-key-row" key={shortcut.keys}>
            <span className="bergag-key">{shortcut.keys}</span>
            <span className="bergag-key-description">{shortcut.description}</span>
            {shortcut.status === "available" ? (
              <span className="bergag-badge bergag-badge--accent">Active</span>
            ) : (
              <span className="bergag-badge">M{shortcut.milestone}</span>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}
