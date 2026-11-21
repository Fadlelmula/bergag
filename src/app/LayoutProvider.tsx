import { useProfile } from "./ProfileProvider";
import { layoutService } from "../application/layoutService";
import { queryKeys } from "../application/queryKeys";
import { DEFAULT_LAYOUT, type BergagLayout } from "../domain/layout";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { createContext, useCallback, useContext, useState, type ReactNode } from "react";

type LayoutUpdater = BergagLayout | ((current: BergagLayout) => BergagLayout);

interface LayoutContextValue {
  layout: BergagLayout;
  isLoading: boolean;
  isSaving: boolean;
  error: string | null;
  updateLayout: (updater: LayoutUpdater, persist?: boolean) => void;
  resetLayout: () => Promise<void>;
}

const LayoutContext = createContext<LayoutContextValue | null>(null);

function resolveUpdater(updater: LayoutUpdater, current: BergagLayout): BergagLayout {
  return typeof updater === "function" ? updater(current) : updater;
}

interface LayoutProviderInnerProps {
  profileId: string;
  children: ReactNode;
}

function LayoutProviderInner({ profileId, children }: LayoutProviderInnerProps) {
  const queryClient = useQueryClient();
  const key = queryKeys.activeLayout(profileId);
  const layoutQuery = useQuery({
    queryKey: key,
    queryFn: layoutService.getActive,
  });
  const [editedLayout, setEditedLayout] = useState<BergagLayout | null>(null);
  const layout = editedLayout ?? layoutQuery.data ?? DEFAULT_LAYOUT;

  const saveMutation = useMutation({
    mutationFn: layoutService.save,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: key }),
  });

  const resetMutation = useMutation({
    mutationFn: layoutService.reset,
    onSuccess: async () => {
      setEditedLayout(structuredClone(DEFAULT_LAYOUT));
      await queryClient.invalidateQueries({ queryKey: key });
    },
  });

  const updateLayout = useCallback(
    (updater: LayoutUpdater, persist = true) => {
      const next = resolveUpdater(updater, layout);
      setEditedLayout(next);
      if (persist) saveMutation.mutate(next);
    },
    [layout, saveMutation],
  );

  const resetLayout = useCallback(async () => {
    await resetMutation.mutateAsync();
  }, [resetMutation]);

  const mutationError = saveMutation.error ?? resetMutation.error;
  const value: LayoutContextValue = {
    layout,
    isLoading: layoutQuery.isLoading,
    isSaving: saveMutation.isPending || resetMutation.isPending,
    error: mutationError
      ? mutationError instanceof Error
        ? mutationError.message
        : String(mutationError)
      : null,
    updateLayout,
    resetLayout,
  };

  return <LayoutContext.Provider value={value}>{children}</LayoutContext.Provider>;
}

export function LayoutProvider({ children }: { children: ReactNode }) {
  const { activeProfile } = useProfile();
  if (!activeProfile) {
    return (
      <LayoutContext.Provider
        value={{
          layout: DEFAULT_LAYOUT,
          isLoading: true,
          isSaving: false,
          error: null,
          updateLayout: () => undefined,
          resetLayout: async () => undefined,
        }}
      >
        {children}
      </LayoutContext.Provider>
    );
  }
  return (
    <LayoutProviderInner key={activeProfile.id} profileId={activeProfile.id}>
      {children}
    </LayoutProviderInner>
  );
}

export function useLayout(): LayoutContextValue {
  const context = useContext(LayoutContext);
  if (!context) throw new Error("useLayout must be used inside LayoutProvider");
  return context;
}
