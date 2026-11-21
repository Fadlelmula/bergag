import { errorMessage } from "../../application/articleService";
import { queryKeys } from "../../application/queryKeys";
import { SwitchField } from "../../components/ui/SwitchField";
import { desktop } from "../../infrastructure/tauri/commands";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

export function StartupSection() {
  const queryClient = useQueryClient();
  const statusQuery = useQuery({
    queryKey: queryKeys.desktopStartup,
    queryFn: desktop.desktopStartupStatus,
  });
  const mutation = useMutation({
    mutationFn: (enabled: boolean) => desktop.desktopStartupSetEnabled(enabled),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.desktopStartup });
    },
  });

  const status = statusQuery.data;

  return (
    <div>
      <p className="bergag-eyebrow">Startup</p>
      <h2 className="bergag-title bergag-title--small">Launch on sign-in</h2>
      <p className="bergag-subtitle">
        This setting changes a Windows sign-in entry for Bergag. Bergag does not add any other
        background service and does not start automatically before you enable this.
      </p>

      {statusQuery.isLoading ? <div className="bergag-loading">Reading startup state…</div> : null}
      {statusQuery.isError ? (
        <div className="bergag-error" role="alert">
          {errorMessage(statusQuery.error)}
        </div>
      ) : null}

      {status ? (
        <div className="bergag-card bergag-card--padded" style={{ marginTop: 18 }}>
          <SwitchField
            label="Launch Bergag when I sign in"
            hint={status.detail}
            checked={status.enabled}
            disabled={!status.available || mutation.isPending}
            onCheckedChange={(enabled) => mutation.mutate(enabled)}
          />
          <div className="bergag-field__hint" role="status" aria-live="polite">
            Launch on sign-in is {status.enabled ? "enabled" : "disabled"}.
          </div>
          {!status.available ? (
            <div className="bergag-field__hint">
              Launch-on-sign-in is unavailable on this platform. No sign-in entry was created.
            </div>
          ) : null}
          {mutation.isError ? (
            <div className="bergag-error" role="alert">
              {errorMessage(mutation.error)}
            </div>
          ) : null}
        </div>
      ) : null}
    </div>
  );
}
