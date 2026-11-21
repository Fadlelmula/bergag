import { render, screen } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { PrivacyDashboard } from "../../domain/models";
import { desktop } from "../../infrastructure/tauri/commands";
import { PrivacySection } from "./PrivacySection";

vi.mock("../../infrastructure/tauri/commands", () => ({
  desktop: { privacyDashboard: vi.fn() },
}));

const mockedDesktop = vi.mocked(desktop);

const report: PrivacyDashboard = {
  app_version: "0.1.0",
  schema_version: 8,
  telemetry_enabled: false,
  ai_enabled: false,
  ai_provider_count: 1,
  remote_ai_provider_count: 1,
  local_ai_provider_count: 0,
  ai_usage_event_count: 3,
  integration_source_count: 2,
  feed_count: 4,
  article_count: 25,
  note_count: 6,
  highlight_count: 7,
  label_count: 8,
  rule_count: 2,
  dashboard_count: 1,
  profile_count: 2,
  backup_count: 3,
  database_path: "C:\\Bergag\\bergag.sqlite3",
  backup_directory: "C:\\Bergag\\backups",
  log_directory: "C:\\Bergag\\logs",
  integrity_check: "ok",
};

function renderSection() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return render(
    <QueryClientProvider client={queryClient}>
      <PrivacySection />
    </QueryClientProvider>,
  );
}

describe("PrivacySection", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockedDesktop.privacyDashboard.mockResolvedValue(report);
  });

  it("renders the real local privacy dashboard without demo values", async () => {
    renderSection();
    expect(await screen.findByText("Schema 8 · ok")).toBeInTheDocument();
    expect(screen.getByText(/Nothing here is a demo value/)).toBeInTheDocument();
    expect(screen.getByText("25")).toBeInTheDocument();
    expect(screen.getByText("2 connected")).toBeInTheDocument();
    expect(screen.getByText("C:\\Bergag\\backups")).toBeInTheDocument();
    expect(screen.getByText("Schema 8 · ok")).toBeInTheDocument();
    expect(mockedDesktop.privacyDashboard).toHaveBeenCalledOnce();
  });

  it("reports telemetry from the app rather than hardcoding it", async () => {
    mockedDesktop.privacyDashboard.mockResolvedValue({ ...report, telemetry_enabled: true });
    renderSection();
    expect(await screen.findByText("Telemetry is enabled in this build.")).toBeInTheDocument();
    expect(screen.getByText("On")).toBeInTheDocument();
  });
});
