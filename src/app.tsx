import { MainDashboardPage } from "./windows/main-dashboard";
import { EdgeWidgetPage } from "./windows/edge-widget";

function currentWindowLabel() {
  if (typeof window === "undefined") {
    return "main-dashboard";
  }

  const label = (window as typeof window & {
    __TAURI_INTERNALS__?: {
      metadata?: {
        currentWindow?: {
          label?: string;
        };
      };
    };
  }).__TAURI_INTERNALS__?.metadata?.currentWindow?.label;

  return typeof label === "string" ? label : "main-dashboard";
}

function App() {
  return currentWindowLabel() === "edge-widget" ? <EdgeWidgetPage /> : <MainDashboardPage />;
}

export default App;
