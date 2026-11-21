import { errorMessage } from "../../application/articleService";
import { queryKeys } from "../../application/queryKeys";
import { Button } from "../../components/ui/Button";
import { desktop } from "../../infrastructure/tauri/commands";
import { useMutation, useQuery } from "@tanstack/react-query";

export function UpdatesSection() {
  const statusQuery = useQuery({
    queryKey: queryKeys.desktopUpdates,
    queryFn: desktop.desktopUpdatesStatus,
  });
  const checkMutation = useMutation({
    mutationFn: desktop.desktopUpdatesCheck,
  });
  const status = statusQuery.data;
  const result = checkMutation.data;

  return (
    <div>
      <p className="bergag-eyebrow">Updates</p>
      <h2 className="bergag-title bergag-title--small">Signed release checks only</h2>
      <p className="bergag-subtitle">
        Bergag never downloads or installs an unverified installer. Checks run only when you ask,
        require a signed HTTPS endpoint compiled into this build, and validate the release
        signature.
      </p>

      {statusQuery.isLoading ? (
        <div className="bergag-loading">Reading update configuration…</div>
      ) : null}
      {statusQuery.isError ? (
        <div className="bergag-error" role="alert">
          {errorMessage(statusQuery.error)}
        </div>
      ) : null}

      {status ? (
        <div className="bergag-card bergag-card--padded" style={{ marginTop: 18 }}>
          <div className="bergag-switch-row">
            <div className="bergag-switch-row__text">
              <span className="bergag-switch-row__label">Current version</span>
              <span className="bergag-switch-row__hint">{status.currentVersion}</span>
            </div>
            <span className="bergag-badge">Local build</span>
          </div>
          <div className="bergag-switch-row">
            <div className="bergag-switch-row__text">
              <span className="bergag-switch-row__label">Signed endpoint</span>
              <span className="bergag-switch-row__hint">
                {status.endpointHost
                  ? `Configured host: ${status.endpointHost}`
                  : "No update endpoint is configured."}
              </span>
            </div>
            <span className="bergag-badge">
              {status.configured ? "Configured" : "Not configured"}
            </span>
          </div>
          <div className="bergag-switch-row">
            <div className="bergag-switch-row__text">
              <span className="bergag-switch-row__label">Signature verification</span>
              <span className="bergag-switch-row__hint">
                {status.signatureVerification
                  ? "Release signatures are verified with the compiled public key."
                  : "No verification key is configured; no release can be trusted."}
              </span>
            </div>
            <span className="bergag-badge">
              {status.signatureVerification ? "Required" : "Unavailable"}
            </span>
          </div>
          <div className="bergag-inline-actions">
            <Button
              size="sm"
              variant="primary"
              disabled={!status.checkAvailable || checkMutation.isPending}
              onClick={() => checkMutation.mutate()}
            >
              {checkMutation.isPending ? "Checking…" : "Check for signed updates"}
            </Button>
          </div>
          <div className="bergag-field__hint">{status.detail}</div>
          <div className="bergag-field__hint">
            Installation is intentionally unavailable in this build. A release must be published and
            signed before installation can be enabled.
          </div>
          {result ? (
            <div className="bergag-note" role="status" aria-live="polite">
              {result.available
                ? `Signed update ${result.version} found at ${result.checkedAt}. ${result.detail}`
                : `No signed update was available at ${result.checkedAt}. ${result.detail}`}
            </div>
          ) : null}
          {checkMutation.isError ? (
            <div className="bergag-error" role="alert">
              {errorMessage(checkMutation.error)}
            </div>
          ) : null}
        </div>
      ) : null}
    </div>
  );
}
