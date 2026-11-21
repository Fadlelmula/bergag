import { invalidateProfileData } from "../application/profileInvalidation";
import { queryKeys } from "../application/queryKeys";
import { profileService } from "../application/profileService";
import type { Profile } from "../domain/models";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { createContext, useContext, type ReactNode } from "react";

interface ProfileContextValue {
  activeProfile: Profile | undefined;
  profiles: Profile[];
  isLoading: boolean;
  error: string | null;
  isSwitching: boolean;
  switchProfile: (id: string) => Promise<void>;
  refresh: () => void;
}

const ProfileContext = createContext<ProfileContextValue | null>(null);

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error ?? "Unknown error");
}

export function ProfileProvider({ children }: { children: ReactNode }) {
  const queryClient = useQueryClient();
  const activeQuery = useQuery({
    queryKey: queryKeys.activeProfile,
    queryFn: profileService.getActive,
  });
  const profilesQuery = useQuery({
    queryKey: queryKeys.profiles,
    queryFn: profileService.list,
  });

  const switchMutation = useMutation({
    mutationFn: profileService.switch,
    onMutate: () => activeQuery.data?.id,
    onSuccess: async (profile, _id, previousProfileId) => {
      await Promise.all([
        invalidateProfileData(queryClient, [previousProfileId, profile.id]),
        queryClient.invalidateQueries({ queryKey: queryKeys.profiles }),
        queryClient.invalidateQueries({ queryKey: queryKeys.activeProfile }),
      ]);
    },
  });

  const refresh = () => {
    void activeQuery.refetch();
    void profilesQuery.refetch();
  };

  const value: ProfileContextValue = {
    activeProfile: activeQuery.data,
    profiles: profilesQuery.data ?? [],
    isLoading: activeQuery.isLoading || profilesQuery.isLoading,
    error:
      activeQuery.error || profilesQuery.error
        ? errorMessage(activeQuery.error ?? profilesQuery.error)
        : null,
    isSwitching: switchMutation.isPending,
    switchProfile: async (id) => {
      await switchMutation.mutateAsync(id);
    },
    refresh,
  };

  return <ProfileContext.Provider value={value}>{children}</ProfileContext.Provider>;
}

export function useProfile(): ProfileContextValue {
  const context = useContext(ProfileContext);
  if (!context) throw new Error("useProfile must be used inside ProfileProvider");
  return context;
}
