import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { ReactNode } from "react";
import { desktop } from "../../infrastructure/tauri/commands";
import { NotificationsSection } from "./NotificationsSection";
import { ProfilesSection } from "./ProfilesSection";
import { SecuritySection } from "./SecuritySection";
import { StartupSection } from "./StartupSection";
import { TraySection } from "./TraySection";
import { UpdatesSection } from "./UpdatesSection";

vi.mock("../../infrastructure/tauri/commands", () => ({
  desktop: {
    desktopNotificationsStatus: vi.fn(),
    desktopNotificationsUpdate: vi.fn(),
    desktopNotificationsTest: vi.fn(),
    desktopTrayStatus: vi.fn(),
    desktopTraySetCloseBehavior: vi.fn(),
    desktopStartupStatus: vi.fn(),
    desktopStartupSetEnabled: vi.fn(),
    desktopUpdatesStatus: vi.fn(),
    desktopUpdatesCheck: vi.fn(),
    desktopSecurityStatus: vi.fn(),
    listProfiles: vi.fn(),
    getActiveProfile: vi.fn(),
  },
}));

const mockedDesktop = vi.mocked(desktop);

function renderWithProviders(node: ReactNode) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  return render(<QueryClientProvider client={queryClient}>{node}</QueryClientProvider>);
}

describe("M9 desktop settings sections", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockedDesktop.desktopNotificationsStatus.mockResolvedValue({
      supported: true,
      permission: "denied",
      settings: {
        enabled: false,
        feedRefresh: true,
        backupComplete: true,
        quietHoursEnabled: false,
        quietHourStart: 22,
        quietHourEnd: 7,
      },
    });
    mockedDesktop.desktopNotificationsUpdate.mockImplementation(async (settings) => settings);
    mockedDesktop.desktopNotificationsTest.mockRejectedValue(new Error("Permission denied"));
    mockedDesktop.desktopTrayStatus.mockResolvedValue({ available: true, closeBehavior: "quit" });
    mockedDesktop.desktopTraySetCloseBehavior.mockResolvedValue({
      available: true,
      closeBehavior: "tray",
    });
    mockedDesktop.desktopStartupStatus.mockResolvedValue({
      available: false,
      enabled: false,
      detail: "Launch-on-sign-in is unavailable on this platform.",
    });
    mockedDesktop.desktopStartupSetEnabled.mockResolvedValue({
      available: false,
      enabled: false,
      detail: "Launch-on-sign-in is unavailable on this platform.",
    });
    mockedDesktop.desktopUpdatesStatus.mockResolvedValue({
      configured: false,
      signatureVerification: false,
      endpointHost: null,
      currentVersion: "0.1.0",
      checkAvailable: false,
      installAvailable: false,
      detail: "No signed update endpoint or verification key is configured.",
    });
    mockedDesktop.desktopSecurityStatus.mockResolvedValue({
      profileLockAvailable: false,
      lockedProfileCount: 0,
      lockDetail: "Bergag does not encrypt or restrict profile access.",
      credentialStore: "Windows Credential Manager (write and clear only; no read command)",
      credentialGetterCommand: false,
      sqliteSecretColumns: 0,
      publicUrlValidation: true,
      htmlSanitization: true,
      logRedaction: true,
      genericShellOrFilesystemAccess: false,
    });
    mockedDesktop.listProfiles.mockResolvedValue([
      {
        id: "p1",
        name: "Reading",
        created_at: "2026-11-20T10:00:00Z",
        updated_at: "2026-11-20T10:00:00Z",
        is_locked: false,
        lock_hint: null,
      },
    ]);
    mockedDesktop.getActiveProfile.mockResolvedValue({
      id: "p1",
      name: "Reading",
      created_at: "2026-11-20T10:00:00Z",
      updated_at: "2026-11-20T10:00:00Z",
      is_locked: false,
      lock_hint: null,
    });
  });

  it("reports real notification permission and refuses a failed test", async () => {
    renderWithProviders(<NotificationsSection />);
    expect(await screen.findByText(/Windows denied notification permission/)).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Send test notification" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("Permission denied");
    expect(mockedDesktop.desktopNotificationsTest).toHaveBeenCalled();
  });

  it("saves notification settings through the narrow command", async () => {
    renderWithProviders(<NotificationsSection />);
    await screen.findByText(/Windows denied notification permission/);
    fireEvent.click(screen.getByRole("switch", { name: "Enable desktop notifications" }));
    fireEvent.click(screen.getByRole("button", { name: "Save notification settings" }));
    await waitFor(() =>
      expect(mockedDesktop.desktopNotificationsUpdate).toHaveBeenCalledWith(
        expect.objectContaining({ enabled: true }),
      ),
    );
  });

  it("changes close behavior with a real command", async () => {
    renderWithProviders(<TraySection />);
    await screen.findByText(/tray icon is enabled/i);
    fireEvent.click(screen.getByRole("radio", { name: "Keep in tray" }));
    await waitFor(() =>
      expect(mockedDesktop.desktopTraySetCloseBehavior).toHaveBeenCalledWith("tray"),
    );
  });

  it("disables startup controls when unavailable", async () => {
    renderWithProviders(<StartupSection />);
    expect(await screen.findAllByText(/unavailable on this platform/i)).not.toHaveLength(0);
    expect(screen.getByRole("switch", { name: "Launch Bergag when I sign in" })).toBeDisabled();
    expect(mockedDesktop.desktopStartupSetEnabled).not.toHaveBeenCalled();
  });

  it("does not offer an update check without a signed endpoint", async () => {
    renderWithProviders(<UpdatesSection />);
    expect(await screen.findByText("Not configured")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Check for signed updates" })).toBeDisabled();
    expect(mockedDesktop.desktopUpdatesCheck).not.toHaveBeenCalled();
  });

  it("shows profile lock as unavailable and reports real security state", async () => {
    renderWithProviders(<SecuritySection />);
    expect(await screen.findAllByText("Unavailable")).not.toHaveLength(0);
    expect(screen.getByText(/does not encrypt or restrict profile access/)).toBeInTheDocument();
    expect(screen.getByText(/No API key, password, token, or secret column/)).toBeInTheDocument();
  });

  it("lists local profiles and keeps the dormant lock flag honest", async () => {
    renderWithProviders(<ProfilesSection />);
    expect(await screen.findByText("Reading")).toBeInTheDocument();
    expect(screen.getByText("Active")).toBeInTheDocument();
    expect(screen.getByText(/does not encrypt or restrict profile access/)).toBeInTheDocument();
  });
});
