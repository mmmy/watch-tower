import { MainDashboardPage } from "./windows/main-dashboard";
import { AlertPopupPage } from "./windows/alert-popup";
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
  const label = currentWindowLabel();

  if (label === "edge-widget") {
    return <EdgeWidgetPage />;
  }

  if (label === "alert-popup" || label.startsWith("alert-popup:")) {
    return <AlertPopupPage />;
  }

  return <MainDashboardPage />;
}

export default App;
