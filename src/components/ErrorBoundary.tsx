import { Component, type ErrorInfo, type ReactNode } from "react";
import i18n from "../i18n";
import { logger } from "../lib/logger";

interface Props {
  children: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

/** Catches React errors, logs them, and renders fallback UI with reload button. */
export default class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    logger.error("Uncaught React error", error, info.componentStack);
  }

  render() {
    if (this.state.hasError) {
      return (
        <div
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            height: "100vh",
            backgroundColor: "#0d1117",
            color: "#c9d1d9",
            fontFamily: "'JetBrains Mono', monospace",
            padding: "2rem",
          }}
        >
          <div style={{ textAlign: "center", maxWidth: 480 }}>
            <h1 style={{ fontSize: "1.25rem", marginBottom: "1rem", color: "#ff7b72" }}>
              {i18n.t("error.somethingWentWrong")}
            </h1>
            <pre
              style={{
                fontSize: "0.75rem",
                padding: "1rem",
                backgroundColor: "#161b22",
                borderRadius: "0.5rem",
                overflow: "auto",
                maxHeight: 200,
                textAlign: "left",
                color: "#8b949e",
              }}
            >
              {this.state.error?.message}
            </pre>
            <button
              type="button"
              onClick={() => window.location.reload()}
              style={{
                marginTop: "1.5rem",
                padding: "0.5rem 1.5rem",
                backgroundColor: "#3b82f6",
                color: "white",
                border: "none",
                borderRadius: "0.375rem",
                cursor: "pointer",
                fontSize: "0.875rem",
              }}
            >
              {i18n.t("error.reloadApplication")}
            </button>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}
