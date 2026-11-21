import { useProfile } from "../../app/ProfileProvider";
import { invalidateProfileData } from "../../application/profileInvalidation";
import { profileService } from "../../application/profileService";
import { Button } from "../../components/ui/Button";
import type { Profile } from "../../domain/models";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";

function messageFrom(error: unknown): string {
  return error instanceof Error ? error.message : String(error ?? "Unknown error");
}

interface PanelProps {
  activeProfile: Profile | undefined;
  profiles: Profile[];
  isSwitching: boolean;
  switchProfile: (id: string) => Promise<void>;
  refresh: () => void;
}

function ProfileSettingsPanel({
  activeProfile,
  profiles,
  isSwitching,
  switchProfile,
  refresh,
}: PanelProps) {
  const queryClient = useQueryClient();
  const [newName, setNewName] = useState("");
  const [renameDraft, setRenameDraft] = useState<string | null>(null);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const renameValue = renameDraft ?? activeProfile?.name ?? "";

  const invalidate = async () => {
    await invalidateProfileData(queryClient, [activeProfile?.id]);
    refresh();
  };

  const createMutation = useMutation({
    mutationFn: profileService.create,
    onSuccess: async (profile) => {
      setNewName("");
      setMessage(`Created profile "${profile.name}". Use the profile selector to switch to it.`);
      await invalidate();
    },
    onError: (error) => setMessage(messageFrom(error)),
  });

  const renameMutation = useMutation({
    mutationFn: (name: string) =>
      profileService.rename(activeProfile ? activeProfile.id : "", name),
    onSuccess: async (profile) => {
      setRenameDraft(null);
      setMessage(`Renamed profile to "${profile.name}".`);
      await invalidate();
    },
    onError: (error) => setMessage(messageFrom(error)),
  });

  const deleteMutation = useMutation({
    mutationFn: () => profileService.remove(activeProfile ? activeProfile.id : ""),
    onSuccess: async () => {
      setMessage("Profile deleted. Bergag switched to a remaining profile.");
      setConfirmDelete(false);
      await invalidate();
    },
    onError: (error) => setMessage(messageFrom(error)),
  });

  const switchMutation = useMutation({
    mutationFn: (id: string) => switchProfile(id),
    onSuccess: async () => {
      setMessage("Active profile changed. Settings and layouts are now profile-isolated.");
      setConfirmDelete(false);
      setRenameDraft(null);
      refresh();
    },
    onError: (error) => setMessage(messageFrom(error)),
  });

  return (
    <div>
      <p className="bergag-eyebrow">Profiles</p>
      <h2 className="bergag-title bergag-title--small">Separate local spaces</h2>
      <p className="bergag-subtitle">
        Every profile has its own settings, layouts, and future library data. Switching does not
        copy content between profiles.
      </p>

      <div style={{ marginTop: 18 }}>
        {profiles.map((profile) => {
          const isActive = profile.id === activeProfile?.id;
          return (
            <div
              key={profile.id}
              className={`bergag-profile-row ${isActive ? "bergag-profile-row--active" : ""}`}
            >
              <div>
                <div className="bergag-profile-row__name">{profile.name}</div>
                <div className="bergag-profile-row__meta">
                  Created {new Date(profile.created_at).toLocaleString()}
                  {isActive ? " · Active" : ""}
                </div>
              </div>
              <Button
                size="sm"
                disabled={isActive || isSwitching || switchMutation.isPending}
                onClick={() => switchMutation.mutate(profile.id)}
              >
                Switch
              </Button>
            </div>
          );
        })}
      </div>

      <h3 className="bergag-section-title">Create profile</h3>
      <div className="bergag-inline-actions">
        <input
          className="bergag-input"
          aria-label="New profile name"
          placeholder="Profile name"
          value={newName}
          maxLength={64}
          onChange={(event) => setNewName(event.target.value)}
        />
        <Button
          variant="primary"
          disabled={newName.trim().length === 0 || createMutation.isPending}
          onClick={() => createMutation.mutate(newName)}
        >
          Create
        </Button>
      </div>

      <h3 className="bergag-section-title">Rename active profile</h3>
      <div className="bergag-inline-actions">
        <input
          className="bergag-input"
          aria-label="Rename active profile"
          value={renameValue}
          maxLength={64}
          onChange={(event) => setRenameDraft(event.target.value)}
        />
        <Button
          disabled={
            !activeProfile ||
            renameValue.trim().length === 0 ||
            renameValue.trim() === activeProfile.name ||
            renameMutation.isPending
          }
          onClick={() => renameMutation.mutate(renameValue)}
        >
          Rename
        </Button>
      </div>

      <h3 className="bergag-section-title">Delete active profile</h3>
      <div className="bergag-note">
        Deleting a profile removes its settings, layouts, and profile-scoped data. Bergag keeps at
        least one profile and creates a verified local backup before deletion, so the library can be
        restored from Settings → Backups if needed.
      </div>
      <div style={{ marginTop: 10 }}>
        {!confirmDelete ? (
          <Button
            variant="danger"
            disabled={!activeProfile || profiles.length <= 1}
            onClick={() => setConfirmDelete(true)}
          >
            Delete “{activeProfile?.name ?? ""}”
          </Button>
        ) : (
          <div className="bergag-inline-actions">
            <span className="bergag-key-description">Delete this profile permanently?</span>
            <Button variant="danger" onClick={() => deleteMutation.mutate()}>
              Confirm delete
            </Button>
            <Button variant="ghost" onClick={() => setConfirmDelete(false)}>
              Cancel
            </Button>
          </div>
        )}
      </div>

      {message ? (
        <p className="bergag-field__hint" role="status" style={{ marginTop: 16 }}>
          {message}
        </p>
      ) : null}
    </div>
  );
}

export function GeneralSection() {
  const profile = useProfile();
  return (
    <ProfileSettingsPanel
      key={profile.activeProfile?.id ?? "none"}
      activeProfile={profile.activeProfile}
      profiles={profile.profiles}
      isSwitching={profile.isSwitching}
      switchProfile={profile.switchProfile}
      refresh={profile.refresh}
    />
  );
}
