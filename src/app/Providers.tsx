import { LayoutProvider } from "./LayoutProvider";
import { ProfileProvider } from "./ProfileProvider";
import { QueryProvider } from "./QueryProvider";
import { ThemeProvider } from "./ThemeProvider";
import type { ReactNode } from "react";

export function Providers({ children }: { children: ReactNode }) {
  return (
    <QueryProvider>
      <ProfileProvider>
        <ThemeProvider>
          <LayoutProvider>{children}</LayoutProvider>
        </ThemeProvider>
      </ProfileProvider>
    </QueryProvider>
  );
}
