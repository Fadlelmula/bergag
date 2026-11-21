import { useProfile } from "../../app/ProfileProvider";
import { errorMessage } from "../../application/articleService";
import { queryKeys } from "../../application/queryKeys";
import { SwitchField } from "../../components/ui/SwitchField";
import type { ArticlePresentation, ReadingSettings } from "../../domain/models";
import { normalizeReading, SETTINGS_KEYS } from "../../domain/settings";
import { desktop } from "../../infrastructure/tauri/commands";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";

const FONT_SIZES = [14, 16, 18, 20, 22] as const;
const LINE_HEIGHTS = [1.4, 1.55, 1.65, 1.8] as const;
const PRESENTATIONS: readonly { value: ArticlePresentation; label: string }[] = [
  { value: "list", label: "List" },
  { value: "magazine", label: "Magazine" },
  { value: "minimal", label: "Minimal" },
];

export function ReadingSection() {
  const { activeProfile } = useProfile();
  const profileId = activeProfile?.id ?? "";
  const queryClient = useQueryClient();
  const [error, setError] = useState<string | null>(null);

  const settings = useQuery({
    queryKey: queryKeys.settings(profileId, SETTINGS_KEYS.reading),
    queryFn: () => desktop.getProfileSetting<ReadingSettings>(SETTINGS_KEYS.reading),
    enabled: Boolean(profileId),
  });
  const reading = normalizeReading(settings.data);

  const mutation = useMutation({
    mutationFn: (patch: Partial<ReadingSettings>) =>
      desktop.setProfileSetting(SETTINGS_KEYS.reading, { ...reading, ...patch }),
    onSuccess: () =>
      queryClient.invalidateQueries({
        queryKey: queryKeys.settings(profileId, SETTINGS_KEYS.reading),
      }),
    onError: (mutationError) => setError(errorMessage(mutationError)),
  });

  const save = (patch: Partial<ReadingSettings>) => {
    setError(null);
    mutation.mutate(patch);
  };

  return (
    <div>
      <p className="bergag-eyebrow">Reading</p>
      <h2 className="bergag-title bergag-title--small">A calm, local reader</h2>
      <p className="bergag-subtitle">
        These preferences are stored per profile. Article text is rendered from sanitized, locally
        stored HTML.
      </p>

      {settings.isLoading ? <div className="bergag-loading">Loading preferences…</div> : null}

      <div className="bergag-card bergag-card--padded" style={{ marginTop: 18 }}>
        <SwitchField
          label="Automatically mark opened articles as read"
          hint="Off by default. Opening an article still records a local history event."
          checked={reading.autoMarkRead}
          disabled={mutation.isPending}
          onCheckedChange={(checked) => save({ autoMarkRead: checked })}
        />
      </div>

      <div className="bergag-field" style={{ marginTop: 18 }}>
        <label className="bergag-field__label" htmlFor="reader-font-size">
          Reader font size
        </label>
        <select
          id="reader-font-size"
          className="bergag-select"
          value={reading.fontSize}
          disabled={mutation.isPending}
          onChange={(event) => save({ fontSize: Number(event.target.value) })}
        >
          {FONT_SIZES.map((size) => (
            <option key={size} value={size}>
              {size} px
            </option>
          ))}
        </select>
      </div>

      <div className="bergag-field">
        <label className="bergag-field__label" htmlFor="reader-line-height">
          Line height
        </label>
        <select
          id="reader-line-height"
          className="bergag-select"
          value={reading.lineHeight}
          disabled={mutation.isPending}
          onChange={(event) => save({ lineHeight: Number(event.target.value) })}
        >
          {LINE_HEIGHTS.map((height) => (
            <option key={height} value={height}>
              {height.toFixed(2)}
            </option>
          ))}
        </select>
      </div>

      <div className="bergag-field">
        <label className="bergag-field__label" htmlFor="article-presentation">
          Default article list layout
        </label>
        <select
          id="article-presentation"
          className="bergag-select"
          value={reading.presentation}
          disabled={mutation.isPending}
          onChange={(event) => save({ presentation: event.target.value as ArticlePresentation })}
        >
          {PRESENTATIONS.map((presentation) => (
            <option key={presentation.value} value={presentation.value}>
              {presentation.label}
            </option>
          ))}
        </select>
      </div>

      <div className="bergag-note">
        Press J and K to move through the list, M to toggle read state, S to save for later, V to
        open the original site, and F to enter focus mode.
      </div>

      {error ? (
        <p className="bergag-error" role="alert" style={{ marginTop: 14 }}>
          {error}
        </p>
      ) : null}
    </div>
  );
}
