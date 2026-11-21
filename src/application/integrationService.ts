import type {
  IntegrationKind,
  IntegrationPlatform,
  IntegrationPreviewResult,
  IntegrationSource,
} from "../domain/models";

export function availableIntegrationPlatforms(
  platforms: IntegrationPlatform[],
): IntegrationPlatform[] {
  return platforms.filter((platform) => platform.available);
}

export function plannedIntegrationPlatforms(
  platforms: IntegrationPlatform[],
): IntegrationPlatform[] {
  return platforms.filter((platform) => !platform.available);
}

export function integrationPlatform(
  platforms: IntegrationPlatform[],
  kind: IntegrationKind,
): IntegrationPlatform | null {
  return platforms.find((platform) => platform.kind === kind) ?? null;
}

export function integrationKindLabel(
  platforms: IntegrationPlatform[],
  kind: IntegrationKind,
): string {
  return integrationPlatform(platforms, kind)?.label ?? kind;
}

export function validateIntegrationSelection(
  kind: IntegrationKind | "",
  input: string,
  platforms: IntegrationPlatform[],
): string | null {
  if (!kind) return "Choose a platform.";
  const platform = integrationPlatform(platforms, kind);
  if (!platform) return "Choose a platform.";
  if (!platform.available) return platform.unavailable_reason ?? "This platform is planned.";
  if (input.trim().length === 0) return `Enter a ${platform.input_label.toLowerCase()}.`;
  return null;
}

export function integrationPreviewNotices(result: IntegrationPreviewResult): string[] {
  const notices: string[] = [];
  if (result.duplicate_source) {
    notices.push(
      result.existing_source
        ? `Already connected as ${result.existing_source.display_name}. Adding this again updates that source.`
        : "This source is already connected in this profile.",
    );
  }
  if (result.duplicate_feed) {
    notices.push("This public feed is already subscribed. Bergag will reuse the existing feed.");
  }
  return notices;
}

export function integrationDeleteMessage(source: IntegrationSource): string {
  return `Remove ${source.display_name} from this profile? The connected feed and its saved articles will remain in your library.`;
}

export function integrationRemoveMessage(source: IntegrationSource): string {
  return `${source.display_name} removed from integrations. The feed and saved articles remain local.`;
}

export interface IntegrationPreviewState {
  kind: IntegrationKind;
  input: string;
  result: IntegrationPreviewResult;
}

export function integrationPreviewIsCurrent(
  preview: IntegrationPreviewState | null,
  kind: IntegrationKind,
  input: string,
): boolean {
  if (!preview) return false;
  return preview.kind === kind && preview.input.trim() === input.trim();
}
