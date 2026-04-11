interface EmptyStateProps {
  title: string;
  body: string;
}

export function EmptyState({ title, body }: EmptyStateProps) {
  return (
    <div className="widget-empty-state">
      <div className="widget-empty-state__title">{title}</div>
      <div className="widget-empty-state__body">{body}</div>
    </div>
  );
}
