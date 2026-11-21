import { errorMessage } from "../../application/articleService";
import { queryKeys } from "../../application/queryKeys";
import { desktop } from "../../infrastructure/tauri/commands";
import { SwitchField } from "../../components/ui/SwitchField";
import { Button } from "../../components/ui/Button";
import type { NotificationSettings } from "../../domain/models";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";

export function NotificationsSection() {
  const queryClient = useQueryClient();
  const statusQuery = useQuery({
    queryKey: queryKeys.desktopNotifications,
    queryFn: desktop.desktopNotificationsStatus,
  });
  const [draft, setDraft] = useState<Partial<NotificationSettings>>({});
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const updateMutation = useMutation({
    mutationFn: (settings: NotificationSettings) => desktop.desktopNotificationsUpdate(settings),
    onSuccess: () => {
      setDraft({});
      setError(null);
      setMessage("Notification preferences were saved locally.");
      void queryClient.invalidateQueries({ queryKey: queryKeys.desktopNotifications });
    },
    onError: (mutationError) => {
      setError(errorMessage(mutationError));
      setMessage(null);
    },
  });

  const testMutation = useMutation({
    mutationFn: desktop.desktopNotificationsTest,
    onSuccess: () => {
      setError(null);
      setMessage("A test notification was sent.");
    },
    onError: (mutationError) => {
      setError(errorMessage(mutationError));
      setMessage(null);
    },
  });

  const status = statusQuery.data;
  const settings = status ? { ...status.settings, ...draft } : undefined;

  const patch = (changes: Partial<NotificationSettings>) => {
    setMessage(null);
    setDraft((current) => ({ ...current, ...changes }));
  };

  const save = () => {
    if (!settings) return;
    updateMutation.mutate(settings);
  };

  return (
    <div>
      <p className="bergag-eyebrow">Notifications</p>
      <h2 className="bergag-title bergag-title--small">Local desktop notifications</h2>
      <p className="bergag-subtitle">
        These controls manage real Windows notifications from this installation. Bergag never sends
        notification content to a server. All preferences are stored locally and are off until you
        enable them.
      </p>

      {statusQuery.isLoading ? (
        <div className="bergag-loading">Reading notification state…</div>
      ) : null}
      {statusQuery.isError ? (
        <div className="bergag-error" role="alert">
          {errorMessage(statusQuery.error)}
        </div>
      ) : null}

      {status && settings ? (
        <>
          <div className="bergag-card bergag-card--padded" style={{ marginTop: 18 }}>
            <div className="bergag-switch-row">
              <div className="bergag-switch-row__text">
                <span className="bergag-switch-row__label">Operating system permission</span>
                <span className="bergag-switch-row__hint">
                  {status.permission === "granted"
                    ? "Windows allows Bergag to show notifications."
                    : status.permission === "denied"
                      ? "Windows denied notification permission. Bergag will not show a notification until you change that permission."
                      : `Current permission state: ${status.permission}.`}
                </span>
              </div>
              <span className="bergag-badge">{status.permission}</span>
            </div>
            <SwitchField
              label="Enable desktop notifications"
              hint="Master switch. No notification is sent while this is off."
              checked={settings.enabled}
              onCheckedChange={(enabled) => patch({ enabled })}
            />
            <SwitchField
              label="Feed refresh finished"
              hint="Notify when a tray Refresh due feeds action finishes."
              checked={settings.feedRefresh}
              onCheckedChange={(feedRefresh) => patch({ feedRefresh })}
              disabled={!settings.enabled}
            />
            <SwitchField
              label="Backup completed"
              hint="Notify when a tray Create local backup action succeeds."
              checked={settings.backupComplete}
              onCheckedChange={(backupComplete) => patch({ backupComplete })}
              disabled={!settings.enabled}
            />
            <SwitchField
              label="Quiet hours"
              hint="Suppress notifications during the window below. Times are local."
              checked={settings.quietHoursEnabled}
              onCheckedChange={(quietHoursEnabled) => patch({ quietHoursEnabled })}
              disabled={!settings.enabled}
            />
            <div className="bergag-inline-actions">
              <label className="bergag-field__label" htmlFor="quiet-hour-start">
                Quiet hours start (0–23)
              </label>
              <input
                id="quiet-hour-start"
                className="bergag-input"
                type="number"
                min={0}
                max={23}
                value={settings.quietHourStart}
                onChange={(event) => patch({ quietHourStart: Number(event.target.value) })}
                disabled={!settings.enabled || !settings.quietHoursEnabled}
              />
              <label className="bergag-field__label" htmlFor="quiet-hour-end">
                End (0–23)
              </label>
              <input
                id="quiet-hour-end"
                className="bergag-input"
                type="number"
                min={0}
                max={23}
                value={settings.quietHourEnd}
                onChange={(event) => patch({ quietHourEnd: Number(event.target.value) })}
                disabled={!settings.enabled || !settings.quietHoursEnabled}
              />
            </div>
            {!status.supported ? (
              <div className="bergag-field__hint" id="notification-support-hint">
                Desktop notifications are not supported in this build. No notification control is
                active.
              </div>
            ) : null}
            <div className="bergag-inline-actions" style={{ marginTop: 12 }}>
              <Button
                size="sm"
                variant="primary"
                disabled={updateMutation.isPending || !status.supported}
                aria-busy={updateMutation.isPending}
                aria-describedby={!status.supported ? "notification-support-hint" : undefined}
                onClick={save}
              >
                Save notification settings
              </Button>
              <Button
                size="sm"
                disabled={testMutation.isPending || !status.supported}
                aria-busy={testMutation.isPending}
                aria-describedby={!status.supported ? "notification-support-hint" : undefined}
                onClick={() => testMutation.mutate()}
              >
                Send test notification
              </Button>
            </div>
            <div className="bergag-field__hint" role="status" aria-live="polite">
              {updateMutation.isPending
                ? "Saving local notification settings…"
                : testMutation.isPending
                  ? "Sending test notification…"
                  : (message ??
                    "The test button reports whether a notification was actually sent. If permission is denied or quiet hours are active, it reports that instead of claiming success.")}
            </div>
            {error ? (
              <div className="bergag-error" role="alert">
                {error}
              </div>
            ) : null}
          </div>
        </>
      ) : null}
    </div>
  );
}
