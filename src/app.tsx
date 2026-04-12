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
  switch (currentWindowLabel()) {
    case "edge-widget":
      return <EdgeWidgetPage />;
    case "alert-popup":
      return <AlertPopupPage />;
    default:
      return <MainDashboardPage />;
  }
}

export default App;
