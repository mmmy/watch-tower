import type { WatchGroupInput } from "../../../shared/config-model";

interface GroupListProps {
  groups: WatchGroupInput[];
  selectedGroupId?: string;
  isSaving: boolean;
  onSelect: (groupId: string) => void;
  onAdd: () => void;
  onDelete: (groupId: string) => void;
}

export function GroupList({
  groups,
  selectedGroupId,
  isSaving,
  onSelect,
  onAdd,
  onDelete,
}: GroupListProps) {
  return (
    <section className="panel section">
      <div className="section__header">
        <div>
          <h3>Watch groups</h3>
          <div className="section__subtle">
            One group maps to one symbol plus one or more signal types.
          </div>
        </div>
        <button className="button button--secondary" type="button" onClick={onAdd} disabled={isSaving}>
          Add group
        </button>
      </div>

      {groups.length === 0 ? (
        <div className="empty-state">
          <div className="empty-state__title">No watch groups yet</div>
          <div className="empty-state__body">
            Create your first group to turn this dashboard back into a monitoring console.
          </div>
        </div>
      ) : (
        <div className="group-list">
          {groups.map((group, index) => {
            const isSelected = group.id === selectedGroupId;

            return (
              <div className={`group-card ${isSelected ? "group-card--active" : ""}`} key={group.id ?? index}>
                <button
                  className="group-card__body"
                  type="button"
                  onClick={() => group.id && onSelect(group.id)}
                  disabled={isSaving || !group.id}
                >
                  <div className="group-card__title">{group.symbol || `Group ${index + 1}`}</div>
                  <div className="group-card__meta">{group.signalTypesText || "No signal types yet"}</div>
                </button>
                <button
                  aria-label={`Delete ${group.symbol || `group ${index + 1}`}`}
                  className="button button--ghost"
                  type="button"
                  onClick={() => group.id && onDelete(group.id)}
                  disabled={isSaving || !group.id}
                >
                  Delete
                </button>
              </div>
            );
          })}
        </div>
      )}
    </section>
  );
}
