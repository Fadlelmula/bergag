import { useTheme } from "../../app/ThemeProvider";
import { SwitchField } from "../../components/ui/SwitchField";
import { ACCENT_PRESETS } from "../../domain/settings";
import type { Density, ThemeMode } from "../../domain/models";
import { useState } from "react";

export function AppearanceSection() {
  const { appearance, resolvedTheme, updateAppearance, isLoading, error } = useTheme();
  const [localError, setLocalError] = useState<string | null>(null);

  const save = async (patch: Parameters<typeof updateAppearance>[0]) => {
    setLocalError(null);
    try {
      await updateAppearance(patch);
    } catch (updateError) {
      setLocalError(
        updateError instanceof Error ? updateError.message : String(updateError ?? "Unknown error"),
      );
    }
  };

  return (
    <div>
      <p className="bergag-eyebrow">Appearance</p>
      <h2 className="bergag-title bergag-title--small">A quiet, editorial desktop</h2>
      <p className="bergag-subtitle">
        Appearance settings are stored per profile. The active theme is {resolvedTheme}.
      </p>
      {isLoading ? <div className="bergag-loading">Loading appearance…</div> : null}

      <div className="bergag-field" style={{ marginTop: 18 }}>
        <label className="bergag-field__label" htmlFor="theme-select">
          Theme
        </label>
        <select
          id="theme-select"
          className="bergag-select"
          value={appearance.theme}
          onChange={(event) => void save({ theme: event.target.value as ThemeMode })}
        >
          <option value="dark">Dark</option>
          <option value="light">Light</option>
          <option value="system">Follow Windows</option>
        </select>
        <span className="bergag-field__hint">Dark is the default. System follows Windows.</span>
      </div>

      <div className="bergag-field">
        <label className="bergag-field__label" htmlFor="accent-color">
          Accent color
        </label>
        <div className="bergag-swatch-row">
          {ACCENT_PRESETS.map((preset) => (
            <button
              key={preset.value}
              type="button"
              className={`bergag-swatch ${appearance.accent === preset.value ? "bergag-swatch--active" : ""}`}
              style={{ background: preset.value }}
              aria-label={`${preset.name} accent`}
              title={preset.name}
              onClick={() => void save({ accent: preset.value })}
            />
          ))}
          <input
            id="accent-color"
            type="color"
            className="bergag-swatch"
            value={appearance.accent}
            aria-label="Custom accent color"
            title="Custom accent color"
            onChange={(event) => void save({ accent: event.target.value })}
          />
        </div>
        <span className="bergag-field__hint">
          Red is used sparingly for selection, focus, and meaningful state.
        </span>
      </div>

      <div className="bergag-field">
        <label className="bergag-field__label" htmlFor="density-select">
          Interface density
        </label>
        <select
          id="density-select"
          className="bergag-select"
          value={appearance.density}
          onChange={(event) => void save({ density: event.target.value as Density })}
        >
          <option value="compact">Compact</option>
          <option value="comfortable">Comfortable</option>
          <option value="spacious">Spacious</option>
        </select>
      </div>

      <SwitchField
        label="Reduce motion"
        hint="Minimizes transitions and animations."
        checked={appearance.reduceMotion}
        onCheckedChange={(checked) => void save({ reduceMotion: checked })}
      />

      {(localError || error) && (
        <p className="bergag-error" role="alert">
          {localError ?? error}
        </p>
      )}
    </div>
  );
}
