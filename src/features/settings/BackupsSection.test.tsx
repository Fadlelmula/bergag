import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { BackupRecord } from "../../domain/models";
import { desktop } from "../../infrastructure/tauri/commands";
import { BackupsSection } from "./BackupsSection";

vi.mock("../../infrastructure/tauri/commands", () => ({
  desktop: {
    backupCreate: vi.fn(),
    backupList: vi.fn(),
    backupVerify: vi.fn(),
    backupDelete: vi.fn(),
    backupRestore: vi.fn(),
  },
}));

const mockedDesktop = vi.mocked(desktop);

const backup: BackupRecord = {
  id: "1e6f4a7c-6b62-4b3e-9c1a-4b8f0f2f5d91",
  label: "Nightly backup",
  created_at: "2026-11-20T12:00:00Z",
  schema_version: 8,
  profile_count: 2,
  size_bytes: 2_359_296,
  automatic: true,
};

function renderSection() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  return render(
    <QueryClientProvider client={queryClient}>
      <BackupsSection />
    </QueryClientProvider>,
  );
}

describe("BackupsSection", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockedDesktop.backupList.mockResolvedValue([backup]);
    mockedDesktop.backupCreate.mockResolvedValue(backup);
    mockedDesktop.backupVerify.mockResolvedValue({ valid: true, reason: null });
    mockedDesktop.backupDelete.mockResolvedValue(undefined);
    mockedDesktop.backupRestore.mockResolvedValue({
      safety_backup_id: "safety-1",
      schema_version: 8,
      profile_count: 2,
    });
  });

  it("lists real backups and explains retention", async () => {
    renderSection();
    expect(await screen.findByText("Nightly backup")).toBeInTheDocument();
    expect(screen.getByText(/2\.3 MB/)).toBeInTheDocument();
    expect(screen.getByText("Automatic")).toBeInTheDocument();
    expect(screen.getByText(/five newest automatic backups are retained/i)).toBeInTheDocument();
  });

  it("creates a verified backup with a user label", async () => {
    renderSection();
    await screen.findByText("Nightly backup");
    fireEvent.change(screen.getByLabelText("Backup label (optional)"), {
      target: { value: "Before trip" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Create verified backup" }));

    await waitFor(() => expect(mockedDesktop.backupCreate).toHaveBeenCalledWith("Before trip"));
    expect(await screen.findByText(/Verified backup .* stored locally/i)).toBeInTheDocument();
  });

  it("shows the real verification result", async () => {
    renderSection();
    await screen.findByText("Nightly backup");
    fireEvent.click(screen.getByRole("button", { name: /Verify backup Nightly backup/i }));

    expect(
      (await screen.findAllByText(/checksum, manifest, profile list, and integrity check/i)).length,
    ).toBeGreaterThan(0);
    expect(mockedDesktop.backupVerify).toHaveBeenCalledWith(backup.id);
  });

  it("confirms destructive delete and restore with explicit wording", async () => {
    mockedDesktop.backupList.mockResolvedValueOnce([backup]).mockResolvedValue([]);
    const confirm = vi.spyOn(window, "confirm").mockReturnValue(true);
    renderSection();
    await screen.findByText("Nightly backup");

    fireEvent.click(screen.getByRole("button", { name: /Delete backup Nightly backup/i }));
    await waitFor(() => expect(mockedDesktop.backupDelete).toHaveBeenCalledWith(backup.id));
    expect(confirm).toHaveBeenCalledWith(expect.stringContaining("removes only the backup files"));

    await waitFor(() => expect(mockedDesktop.backupList).toHaveBeenCalledTimes(2));
    confirm.mockRestore();
  });

  it("warns that restore replaces the library and creates a safety backup", async () => {
    const confirm = vi.spyOn(window, "confirm").mockReturnValue(true);
    renderSection();
    await screen.findByText("Nightly backup");

    fireEvent.click(screen.getByRole("button", { name: /Restore backup Nightly backup/i }));
    await waitFor(() => expect(mockedDesktop.backupRestore).toHaveBeenCalledWith(backup.id));
    expect(confirm).toHaveBeenCalledWith(
      expect.stringContaining("current library will be replaced"),
    );
    expect(await screen.findByText(/safety backup was created first/i)).toBeInTheDocument();
    confirm.mockRestore();
  });
});
