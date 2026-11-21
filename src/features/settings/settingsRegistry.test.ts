import { describe, expect, it } from "vitest";
import { renderSection } from "./SettingsDialog";
import { SETTINGS_SECTIONS } from "./settingsRegistry";

describe("settings registry", () => {
  it("has unique section ids", () => {
    const ids = SETTINGS_SECTIONS.map((section) => section.id);
    expect(new Set(ids).size).toBe(ids.length);
  });

  it("provides a real renderer for every available section", () => {
    for (const section of SETTINGS_SECTIONS.filter((item) => item.available)) {
      expect(renderSection(section.id), `${section.id} should have a renderer`).not.toBeNull();
    }
  });

  it("does not render a body for planned sections", () => {
    for (const section of SETTINGS_SECTIONS.filter((item) => !item.available)) {
      expect(renderSection(section.id), `${section.id} should be planned`).toBeNull();
    }
  });

  it("marks unfinished milestone 9 areas as planned", () => {
    const planned = new Set(
      SETTINGS_SECTIONS.filter((section) => !section.available).map((section) => section.id),
    );
    expect(planned).toEqual(new Set(["advanced"]));
  });

  it("does not keep stale planned feed or refresh settings", () => {
    const ids = SETTINGS_SECTIONS.map((section) => section.id);
    expect(ids).not.toContain("feeds");
    expect(ids).not.toContain("refresh");
  });
});
