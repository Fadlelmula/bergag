import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { Diagnostics } from "../../domain/models";
import { FoundationView } from "./FoundationView";

const diagnostics: Diagnostics = {
  app_version: "0.1.0",
  tauri_version: "2",
  os: "windows",
  arch: "x86_64",
  telemetry_enabled: false,
  log_dir: "C:/logs",
  database: {
    path: "C:/data/bergag.sqlite3",
    size_bytes: 2048,
    sqlite_version: "3.50.0",
    migration_count: 1,
    user_version: 1,
    integrity_check: "ok",
    fts5_available: true,
  },
};

describe("FoundationView", () => {
  it("renders real diagnostics without fabricating library metrics", () => {
    render(
      <FoundationView
        activeProfile={{
          id: "1",
          name: "Default",
          created_at: "",
          updated_at: "",
          is_locked: false,
          lock_hint: null,
        }}
        profileCount={1}
        diagnostics={diagnostics}
        diagnosticsLoading={false}
        diagnosticsError={null}
        onOpenSettings={vi.fn()}
      />,
    );

    expect(screen.getByText("ok")).toBeInTheDocument();
    expect(screen.getByText("3.50.0")).toBeInTheDocument();
    expect(screen.getByText("1 profile stored locally")).toBeInTheDocument();
    expect(screen.queryByText(/unread/i)).not.toBeInTheDocument();
  });

  it("opens privacy settings from the real action", () => {
    const onOpenSettings = vi.fn();
    render(
      <FoundationView
        activeProfile={undefined}
        profileCount={0}
        diagnostics={undefined}
        diagnosticsLoading={false}
        diagnosticsError={null}
        onOpenSettings={onOpenSettings}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: /privacy and diagnostics/i }));
    expect(onOpenSettings).toHaveBeenCalled();
  });
});
