import { DiagnosticsPanel } from "./DiagnosticsPanel";

export function AboutSection() {
  return (
    <div>
      <p className="bergag-eyebrow">About</p>
      <h2 className="bergag-title bergag-title--small">Bergag Foundation</h2>
      <p className="bergag-subtitle">
        A local-first reading and research environment for Windows. Core functionality never
        requires an account, cloud service, or AI provider.
      </p>
      <DiagnosticsPanel />
    </div>
  );
}
