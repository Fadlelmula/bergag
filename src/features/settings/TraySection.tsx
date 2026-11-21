import { errorMessage } from "../../application/articleService";
import { queryKeys } from "../../application/queryKeys";
import { Button } from "../../components/ui/Button";
import { desktop } from "../../infrastructure/tauri/commands";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

export function TraySection() {
  const queryClient = useQueryClient();
  const statusQuery = useQuery({
    queryKey: queryKeys.desktopTray,
    queryFn: desktop.desktopTrayStatus,
  });
  const mutation = useMutation({
    mutationFn: (behavior: "quit" | "tray") => desktop.desktopTraySetCloseBehavior(behavior),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.desktopTray });
    },
  });

  const status = statusQuery.data;
  const behavior = status?.closeBehavior ?? "quit";
  const closeBehaviorLabel = behavior === "tray" ? "Keep in tray" : "Quit Bergag";

  return (
    <div>
      <p className="bergag-eyebrow">System tray</p>
      <h2 className="bergag-title bergag-title--small">Tray controls and close behavior</h2>
      <p className="bergag-subtitle">
        When available, the tray icon provides Show Bergag, Refresh due feeds, Create local backup,
        and Quit Bergag. Each action performs the same real work as its in-app equivalent.
      </p>

      {statusQuery.isLoading ? <div className="bergag-loading">Reading tray state…</div> : null}
      {statusQuery.isError ? (
        <div className="bergag-error" role="alert">
          {errorMessage(statusQuery.error)}
        </div>
      ) : null}

      {status ? (
        <div className="bergag-card bergag-card--padded" style={{ marginTop: 18 }}>
          <div className="bergag-switch-row">
            <div className="bergag-switch-row__text">
              <span className="bergag-switch-row__label">Tray icon</span>
              <span className="bergag-switch-row__hint">
                {status.available
                  ? "The tray icon is enabled in this Windows build."
                  : "A system tray is not available in this build."}
              </span>
            </div>
            <span className="bergag-badge">{status.available ? "Available" : "Unavailable"}</span>
          </div>
          <div className="bergag-field">
            <span className="bergag-field__label" id="tray-close-behavior-label">
              When the window is closed
            </span>
            <div
              className="bergag-inline-actions"
              role="radiogroup"
              aria-labelledby="tray-close-behavior-label"
            >
              <Button
                size="sm"
                variant={behavior === "quit" ? "primary" : "secondary"}
                role="radio"
                aria-checked={behavior === "quit"}
                disabled={mutation.isPending}
                onClick={() => mutation.mutate("quit")}
              >
                Quit Bergag
              </Button>
              <Button
                size="sm"
                variant={behavior === "tray" ? "primary" : "secondary"}
                role="radio"
                aria-checked={behavior === "tray"}
                disabled={mutation.isPending || !status.available}
                onClick={() => mutation.mutate("tray")}
              >
                Keep in tray
              </Button>
            </div>
            <div className="bergag-field__hint" role="status" aria-live="polite">
              Close behavior: {closeBehaviorLabel}.
            </div>
            <div className="bergag-field__hint">
              “Keep in tray” hides the window on close; Bergag keeps running until you choose Quit
              Bergag in the tray menu. This preference is stored locally and has no effect if the
              tray is unavailable.
            </div>
          </div>
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
