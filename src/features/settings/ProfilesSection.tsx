import { errorMessage } from "../../application/articleService";
import { queryKeys } from "../../application/queryKeys";
import { desktop } from "../../infrastructure/tauri/commands";
import { useQuery } from "@tanstack/react-query";

export function ProfilesSection() {
  const profilesQuery = useQuery({
    queryKey: queryKeys.profiles,
    queryFn: desktop.listProfiles,
  });
  const activeQuery = useQuery({
    queryKey: queryKeys.activeProfile,
    queryFn: desktop.getActiveProfile,
  });
  const profiles = profilesQuery.data ?? [];
  const activeId = activeQuery.data?.id;
  const activeStateKnown = activeQuery.isSuccess && !activeQuery.isError;
  const locked = profiles.filter((profile) => profile.is_locked).length;

  return (
    <div>
      <p className="bergag-eyebrow">Profiles</p>
      <h2 className="bergag-title bergag-title--small">Local profile state</h2>
      <p className="bergag-subtitle">
        This page reports the profiles stored in this installation. Creating, renaming, switching,
        and deleting profiles are handled in General. Profile data is isolated by profile ID.
      </p>

      {profilesQuery.isLoading ? <div className="bergag-loading">Reading profiles…</div> : null}
      {profilesQuery.isError ? (
        <div className="bergag-error" role="alert">
          {errorMessage(profilesQuery.error)}
        </div>
      ) : null}
      {activeQuery.isError ? (
        <div className="bergag-error" role="alert">
          {errorMessage(activeQuery.error)}
        </div>
      ) : null}

      <div className="bergag-card bergag-card--padded" style={{ marginTop: 18 }}>
        {profiles.map((profile) => (
          <div className="bergag-switch-row" key={profile.id}>
            <div className="bergag-switch-row__text">
              <span className="bergag-switch-row__label">{profile.name}</span>
              <span className="bergag-switch-row__hint">
                Created {profile.created_at} ·{" "}
                {profile.is_locked ? "dormant lock flag set" : "no lock flag"}
              </span>
            </div>
            <span className="bergag-badge">
              {!activeStateKnown ? "Unknown" : profile.id === activeId ? "Active" : "Inactive"}
            </span>
          </div>
        ))}
        {!profilesQuery.isLoading && profiles.length === 0 ? (
          <div className="bergag-note">No profiles were returned by this installation.</div>
        ) : null}
      </div>

      <div className="bergag-card bergag-card--padded">
        <div className="bergag-switch-row">
          <div className="bergag-switch-row__text">
            <span className="bergag-switch-row__label">Profile lock</span>
            <span className="bergag-switch-row__hint">
              Unavailable. {locked} dormant lock flag(s) exist in the database, but Bergag does not
              encrypt or restrict profile access. No lock or unlock control is offered.
            </span>
          </div>
          <span className="bergag-badge">Unavailable</span>
        </div>
        <div className="bergag-switch-row">
          <div className="bergag-switch-row__text">
            <span className="bergag-switch-row__label">Profile export</span>
            <span className="bergag-switch-row__hint">
              Use Settings → Backups to create a verified local snapshot of the whole library. There
              is no separate per-profile export command.
            </span>
          </div>
          <span className="bergag-badge">Backups</span>
        </div>
      </div>
    </div>
  );
}
