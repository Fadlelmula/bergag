import { Component, type ErrorInfo, type ReactNode } from "react";

interface State {
  hasError: boolean;
}

export class ErrorBoundary extends Component<{ children: ReactNode }, State> {
  state: State = { hasError: false };

  static getDerivedStateFromError(): State {
    return { hasError: true };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error("Bergag UI error", error.message, info.componentStack);
  }

  render() {
    if (!this.state.hasError) return this.props.children;
    return (
      <div className="bergag-empty-state" style={{ height: "100%" }}>
        <div className="bergag-empty-state__mark">!</div>
        <h1 className="bergag-empty-state__title">Bergag hit an unexpected UI error</h1>
        <p className="bergag-empty-state__text">
          Restart the application. Your local database is not reset automatically. If this keeps
          happening, report the issue without including private article content.
        </p>
      </div>
    );
  }
}
