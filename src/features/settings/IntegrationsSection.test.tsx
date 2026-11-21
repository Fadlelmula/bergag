import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type {
  IntegrationPlatform,
  IntegrationPreviewResult,
  IntegrationSource,
} from "../../domain/models";
import { desktop } from "../../infrastructure/tauri/commands";
import { IntegrationsSection } from "./IntegrationsSection";

vi.mock("../../app/ProfileProvider", () => ({
  useProfile: () => ({ activeProfile: { id: "profile-1" } }),
}));

vi.mock("../../infrastructure/tauri/commands", () => ({
  desktop: {
    integrationPlatforms: vi.fn(),
    integrationPreview: vi.fn(),
    integrationAdd: vi.fn(),
    integrationList: vi.fn(),
    integrationSetTrust: vi.fn(),
    integrationRefresh: vi.fn(),
    integrationDelete: vi.fn(),
  },
}));

const mockedDesktop = vi.mocked(desktop);

const platforms: IntegrationPlatform[] = [
  {
    kind: "youtube",
    label: "YouTube",
    description: "Public channel feed",
    input_label: "Channel ID",
    input_hint: "A 24-character ID beginning with UC.",
    available: true,
    unavailable_reason: null,
  },
  {
    kind: "podcast",
    label: "Podcast",
    description: "Public RSS feed",
    input_label: "Feed URL",
    input_hint: "A public RSS URL.",
    available: true,
    unavailable_reason: null,
  },
  {
    kind: "bluesky",
    label: "Bluesky",
    description: "Planned",
    input_label: "Handle",
    input_hint: "Not available.",
    available: false,
    unavailable_reason: "Requires an authenticated API; planned.",
  },
];

const source: IntegrationSource = {
  id: "source-1",
  kind: "youtube",
  source_key: "UCabcdefghijklmnopqrstuv",
  display_name: "Example channel",
  source_url: null,
  feed_url: "https://www.youtube.com/feeds/videos.xml?channel_id=UCabcdefghijklmnopqrstuv",
  feed_id: "feed-1",
  category: null,
  trust_level: "untrusted",
  status: "active",
  last_error: null,
  created_at: "2026-01-01T00:00:00Z",
  updated_at: "2026-01-01T00:00:00Z",
};

function previewResult(
  overrides: Partial<IntegrationPreviewResult> = {},
): IntegrationPreviewResult {
  return {
    platform: platforms[0]!,
    resolved_feed_url: source.feed_url,
    preview: {
      title: "Example channel",
      site_url: null,
      feed_type: "atom",
      description: null,
      language: null,
      item_count: 3,
      items: [],
    },
    duplicate_feed: false,
    duplicate_source: false,
    existing_source: null,
    ...overrides,
  };
}

function renderSection() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  return render(
    <QueryClientProvider client={queryClient}>
      <IntegrationsSection />
    </QueryClientProvider>,
  );
}

describe("IntegrationsSection", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockedDesktop.integrationPlatforms.mockResolvedValue(platforms);
    mockedDesktop.integrationList.mockResolvedValue([]);
    mockedDesktop.integrationPreview.mockResolvedValue(previewResult());
    mockedDesktop.integrationAdd.mockResolvedValue(source);
    mockedDesktop.integrationSetTrust.mockResolvedValue(source);
    mockedDesktop.integrationRefresh.mockResolvedValue([]);
    mockedDesktop.integrationDelete.mockResolvedValue(undefined);
  });

  it("shows real availability and keeps planned platforms disabled", async () => {
    renderSection();
    await screen.findByText("YouTube");
    const planned = screen.getByRole("option", { name: "Bluesky (planned)" }) as HTMLOptionElement;
    expect(planned.disabled).toBe(true);
    expect(screen.getByText("1 planned")).toBeInTheDocument();
    expect(screen.getByText("2")).toBeInTheDocument();
  });

  it("renders a real preview before allowing add", async () => {
    renderSection();
    await screen.findByText("YouTube");
    fireEvent.change(screen.getByLabelText("Channel ID"), {
      target: { value: "UCabcdefghijklmnopqrstuv" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Preview feed" }));

    expect(await screen.findByText("Example channel")).toBeInTheDocument();
    expect(screen.getByText(source.feed_url)).toBeInTheDocument();
    expect(screen.getByText("3 feed items")).toBeInTheDocument();
    expect(mockedDesktop.integrationPreview).toHaveBeenCalledWith(
      "youtube",
      "UCabcdefghijklmnopqrstuv",
    );
    expect(screen.getByRole("button", { name: "Add & subscribe" })).toBeEnabled();
  });

  it("shows duplicate notices from the preview result", async () => {
    mockedDesktop.integrationPreview.mockResolvedValue(
      previewResult({ duplicate_source: true, duplicate_feed: true, existing_source: source }),
    );
    renderSection();
    await screen.findByText("YouTube");
    fireEvent.change(screen.getByLabelText("Channel ID"), {
      target: { value: "UCabcdefghijklmnopqrstuv" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Preview feed" }));

    expect(
      await screen.findByText(
        "Already connected as Example channel. Adding this again updates that source.",
      ),
    ).toBeInTheDocument();
    expect(
      screen.getByText(
        "This public feed is already subscribed. Bergag will reuse the existing feed.",
      ),
    ).toBeInTheDocument();
  });

  it("refreshes the connected sources after a removal and explains what is kept", async () => {
    mockedDesktop.integrationList.mockResolvedValueOnce([source]).mockResolvedValue([]);
    const confirm = vi.spyOn(window, "confirm").mockReturnValue(true);
    renderSection();

    expect(await screen.findByText("Example channel")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Remove" }));

    await waitFor(() => expect(mockedDesktop.integrationDelete).toHaveBeenCalledWith("source-1"));
    expect(confirm).toHaveBeenCalledWith(
      "Remove Example channel from this profile? The connected feed and its saved articles will remain in your library.",
    );
    await waitFor(() => expect(mockedDesktop.integrationList).toHaveBeenCalledTimes(2));
    expect(
      await screen.findByText(
        "Example channel removed from integrations. The feed and saved articles remain local.",
      ),
    ).toBeInTheDocument();
    confirm.mockRestore();
  });
});
