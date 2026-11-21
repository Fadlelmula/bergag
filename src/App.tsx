import { ErrorBoundary } from "./app/ErrorBoundary";
import { Providers } from "./app/Providers";
import { SpiderLilyMark } from "./components/branding/SpiderLilyMark";
import { AppShell } from "./features/shell/AppShell";
import { isTauriRuntime } from "./infrastructure/tauri/client";

function RuntimeUnavailable() {
  return (
    <div className="bergag-empty-state" style={{ height: "100%" }}>
      <div className="bergag-empty-state__mark">
        <SpiderLilyMark size={54} />
      </div>
      <h1 className="bergag-empty-state__title">Desktop runtime required</h1>
      <p className="bergag-empty-state__text">
        Bergag's local database and native commands are only available in the Tauri desktop shell.
        Launch it with <code>npm run tauri dev</code> or an installed build.
      </p>
    </div>
  );
}

export default function App() {
  if (!isTauriRuntime()) {
    return <RuntimeUnavailable />;
  }
  return (
    <ErrorBoundary>
      <Providers>
        <AppShell />
      </Providers>
    </ErrorBoundary>
  );
}
