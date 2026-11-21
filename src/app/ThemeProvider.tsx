import { useProfile } from "./ProfileProvider";
import { queryKeys } from "../application/queryKeys";
import { DEFAULT_APPEARANCE, normalizeAppearance, SETTINGS_KEYS } from "../domain/settings";
import type { AppearanceSettings } from "../domain/models";
import { desktop } from "../infrastructure/tauri/commands";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { createContext, useContext, useEffect, useMemo, useState, type ReactNode } from "react";

type SystemTheme = "dark" | "light";

interface ThemeContextValue {
  appearance: AppearanceSettings;
  resolvedTheme: SystemTheme;
  isLoading: boolean;
  error: string | null;
  updateAppearance: (patch: Partial<AppearanceSettings>) => Promise<void>;
}

const ThemeContext = createContext<ThemeContextValue | null>(null);

function useSystemTheme(): SystemTheme {
  const [theme, setTheme] = useState<SystemTheme>(() => {
    if (typeof window === "undefined" || !window.matchMedia) return "dark";
    return window.matchMedia("(prefers-color-scheme: light)").matches ? "light" : "dark";
  });

  useEffect(() => {
    if (!window.matchMedia) return;
    const query = window.matchMedia("(prefers-color-scheme: light)");
    const listener = (event: MediaQueryListEvent) => setTheme(event.matches ? "light" : "dark");
    query.addEventListener("change", listener);
    return () => query.removeEventListener("change", listener);
  }, []);

  return theme;
}

export function ThemeProvider({ children }: { children: ReactNode }) {
  const { activeProfile } = useProfile();
  const queryClient = useQueryClient();
  const systemTheme = useSystemTheme();
  const key = activeProfile ? queryKeys.appearance(activeProfile.id) : ["settings", "none"];

  const settingQuery = useQuery({
    queryKey: key,
    queryFn: () => desktop.getProfileSetting<unknown>(SETTINGS_KEYS.appearance),
    enabled: Boolean(activeProfile),
  });

  const appearance = useMemo(
    () => normalizeAppearance(settingQuery.data ?? DEFAULT_APPEARANCE),
    [settingQuery.data],
  );
  const resolvedTheme: SystemTheme = appearance.theme === "system" ? systemTheme : appearance.theme;

  const updateMutation = useMutation({
    mutationFn: (next: AppearanceSettings) =>
      desktop.setProfileSetting(SETTINGS_KEYS.appearance, next),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: key }),
  });

  useEffect(() => {
    const root = document.documentElement;
    root.dataset.theme = resolvedTheme;
    root.dataset.density = appearance.density;
    root.dataset.reduceMotion = String(appearance.reduceMotion);
    root.style.setProperty("--accent", appearance.accent);
  }, [appearance.accent, appearance.density, appearance.reduceMotion, resolvedTheme]);

  const value: ThemeContextValue = {
    appearance,
    resolvedTheme,
    isLoading: settingQuery.isLoading,
    error: settingQuery.error
      ? settingQuery.error instanceof Error
        ? settingQuery.error.message
        : String(settingQuery.error)
      : null,
    updateAppearance: async (patch) => {
      await updateMutation.mutateAsync({ ...appearance, ...patch });
    },
  };

  return <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>;
}

export function useTheme(): ThemeContextValue {
  const context = useContext(ThemeContext);
  if (!context) throw new Error("useTheme must be used inside ThemeProvider");
  return context;
}
