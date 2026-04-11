import type { ReactNode } from "react";

interface DashboardShellProps {
  sidebar: ReactNode;
  main: ReactNode;
  aside: ReactNode;
}

export function DashboardShell({ sidebar, main, aside }: DashboardShellProps) {
  return (
    <section className="dashboard-shell">
      <div className="dashboard-shell__sidebar">{sidebar}</div>
      <div className="dashboard-shell__main">{main}</div>
      <div className="dashboard-shell__aside">{aside}</div>
    </section>
  );
}
