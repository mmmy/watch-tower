import { useEffect, useMemo, useState } from "react";
import {
  buildDashboardRecoveryViewModel,
  buildGroupViewModel,
  getSnapshotRuntimeStatus,
} from "../../shared/view-models";
import {
  createWatchGroupInput,
  toConfigInput,
  type AppConfigInput,
} from "../../shared/config-model";
import { BootstrapPanel } from "./components/bootstrap-panel";
import { ConfigSummary } from "./components/config-summary";
import { DashboardShell } from "./components/dashboard-shell";
import { DiagnosticsPanel } from "./components/diagnostics-panel";
import { GroupEditor } from "./components/group-editor";
import { GroupList } from "./components/group-list";
import { LayoutPresetToggle } from "./components/layout-preset-toggle";
import { PeriodMatrixDebug } from "./components/period-matrix-debug";
import { PollingHealthPanel } from "./components/polling-health-panel";
import { RecoveryPanel } from "./components/recovery-panel";
import { Timeline60Debug } from "./components/timeline-60-debug";
import { WindowPolicyForm } from "./components/window-policy-form";
import { useAppEvents } from "./hooks/use-app-events";

export function MainDashboardPage() {
  const {
    snapshot,
    isSaving,
    submitError,
    saveConfig,
    pollNow,
    selectGroup,
    markAlertRead,
    openAlertInDashboard,
    clearDashboardFocusIntent,
  } = useAppEvents();
  const [activePeriod, setActivePeriod] = useState<string | undefined>(undefined);
  const [activeSignalType, setActiveSignalType] = useState<string | undefined>(undefined);
  const [draftConfig, setDraftConfig] = useState<AppConfigInput | null>(null);

  const configSyncKey = useMemo(
    () => (snapshot?.config ? JSON.stringify(snapshot.config) : "bootstrap"),
    [snapshot?.config],
  );

  useEffect(() => {
    if (!snapshot?.config) {
      setDraftConfig(null);
      return;
    }

    setDraftConfig(toConfigInput(snapshot.config));
  }, [configSyncKey, snapshot?.config]);

  useEffect(() => {
    const focusIntent = snapshot?.alertRuntime.dashboardFocusIntent;

    if (!focusIntent) {
      return;
    }

    setActivePeriod(focusIntent.alert.period);
    setActiveSignalType(focusIntent.alert.signalType);
    void clearDashboardFocusIntent();
  }, [clearDashboardFocusIntent, snapshot?.alertRuntime.dashboardFocusIntent]);

  const viewModel = useMemo(
    () => (snapshot ? buildGroupViewModel(snapshot, activePeriod, activeSignalType) : null),
    [snapshot, activePeriod, activeSignalType],
  );
  const recoveryItems = useMemo(
    () => (snapshot ? buildDashboardRecoveryViewModel(snapshot) : []),
    [snapshot],
  );

  const selectedDraftGroup =
    draftConfig?.groups?.find((group) => group.id === draftConfig.selectedGroupId) ??
    draftConfig?.groups?.[0] ??
    null;

  async function commitDraft(nextDraft: AppConfigInput) {
    setDraftConfig(nextDraft);
    await saveConfig(nextDraft);
  }

  function replaceCurrentGroup(nextGroup: NonNullable<typeof selectedDraftGroup>) {
    setDraftConfig((currentDraft) => {
      if (!currentDraft) {
        return currentDraft;
      }

      return {
        ...currentDraft,
        groups: (currentDraft.groups ?? []).map((group) =>
          group.id === nextGroup.id ? nextGroup : group,
        ),
      };
    });
  }

  function nextGroupId(input: AppConfigInput) {
    const takenIds = new Set((input.groups ?? []).map((group) => group.id).filter(Boolean));
    let suffix = (input.groups?.length ?? 0) + 1;
    let candidate = `group-${suffix}`;

    while (takenIds.has(candidate)) {
      suffix += 1;
      candidate = `group-${suffix}`;
    }

    return candidate;
  }

  return (
    <main className="shell">
      <div className="dashboard">
        <section className="dashboard__hero">
          <div className="panel hero-card">
            <span className="hero-card__eyebrow">Watch Tower v0.2</span>
            <div>
              <h1>Main dashboard, now ready for resident desktop work.</h1>
              <p>
                The foundation is already proven. This pass turns the verification shell into a
                real control console for multi-group configuration, current-group focus, and the
                minimum window policy needed before widget and tray work begins.
              </p>
            </div>
            <div className="hero-card__meta">
              <div className="hero-meta">
                <div className="hero-meta__label">Current mode</div>
                <div className="hero-meta__value">
                  {snapshot?.bootstrapRequired ? "Bootstrap required" : "Verification shell"}
                </div>
              </div>
              <div className="hero-meta">
                <div className="hero-meta__label">Focus</div>
                <div className="hero-meta__value">Groups, selection, layout, and runtime policy</div>
              </div>
              <div className="hero-meta">
                <div className="hero-meta__label">Runtime</div>
                <div className="hero-meta__value">{snapshot?.health.status ?? "loading"}</div>
              </div>
            </div>
          </div>

          {snapshot?.bootstrapRequired || !draftConfig ? (
            <BootstrapPanel
              initialValues={snapshot?.config ? toConfigInput(snapshot.config) : undefined}
              isSaving={isSaving}
              submitError={submitError}
              onSubmit={saveConfig}
            />
          ) : null}
        </section>

        {snapshot && draftConfig ? (
          <DashboardShell
            sidebar={
              <GroupList
                groups={draftConfig.groups ?? []}
                selectedGroupId={draftConfig.selectedGroupId}
                isSaving={isSaving}
                onSelect={async (groupId) => {
                  setDraftConfig((currentDraft) =>
                    currentDraft ? { ...currentDraft, selectedGroupId: groupId } : currentDraft,
                  );
                  setActivePeriod(undefined);
                  setActiveSignalType(undefined);
                  await selectGroup(groupId);
                }}
                onAdd={async () => {
                  const nextDraft = {
                    ...draftConfig,
                    groups: [
                      ...(draftConfig.groups ?? []),
                      createWatchGroupInput({
                        id: nextGroupId(draftConfig),
                        symbol: selectedDraftGroup?.symbol ?? "BTCUSDT",
                        signalTypesText: selectedDraftGroup?.signalTypesText ?? "vegas,divMacd",
                        periods: selectedDraftGroup?.periods,
                        selectedTimelinePeriod: selectedDraftGroup?.selectedTimelinePeriod,
                      }),
                    ],
                  };
                  const nextGroup = nextDraft.groups?.[nextDraft.groups.length - 1];

                  await commitDraft({
                    ...nextDraft,
                    selectedGroupId: nextGroup?.id,
                  });
                }}
                onDelete={async (groupId) => {
                  const nextGroups = (draftConfig.groups ?? []).filter((group) => group.id !== groupId);
                  await commitDraft({
                    ...draftConfig,
                    groups: nextGroups,
                    selectedGroupId:
                      draftConfig.selectedGroupId === groupId
                        ? nextGroups[0]?.id ?? ""
                        : draftConfig.selectedGroupId,
                  });
                }}
              />
            }
            main={
              <div className="stack">
                <ConfigSummary config={snapshot.config!} />
                <section className="panel section">
                  <div className="section__header">
                    <div>
                      <h3>Current group detail</h3>
                      <div className="section__subtle">
                        Scan the selected group, then drill into one period and signal type.
                      </div>
                    </div>
                    <LayoutPresetToggle
                      value={draftConfig.layoutPreset ?? "table"}
                      onChange={(layoutPreset) =>
                        setDraftConfig((currentDraft) =>
                          currentDraft ? { ...currentDraft, layoutPreset } : currentDraft,
                        )
                      }
                    />
                  </div>

                  {viewModel ? (
                    <div className={`dashboard-detail dashboard-detail--${draftConfig.layoutPreset ?? "table"}`}>
                      <PeriodMatrixDebug
                        activePeriod={viewModel.activePeriod}
                        activeSignalType={viewModel.activeSignalType}
                        snapshot={viewModel.groupSnapshot}
                        onSelect={(period, signalType) => {
                          setActivePeriod(period);
                          setActiveSignalType(signalType);
                        }}
                      />
                      <Timeline60Debug
                        period={viewModel.activePeriod}
                        signal={viewModel.selectedSignal}
                        signalType={viewModel.activeSignalType}
                      />
                    </div>
                  ) : (
                    <div className="empty-state">
                      <div className="empty-state__title">No active watch group</div>
                      <div className="empty-state__body">
                        Create a group on the left to restore matrix and timeline monitoring for a
                        single symbol.
                      </div>
                    </div>
                  )}
                </section>
              </div>
            }
            aside={
              <div className="stack">
                <BootstrapPanel
                  initialValues={draftConfig}
                  isSaving={isSaving}
                  submitError={submitError}
                  heading="Runtime settings"
                  subtle="Keep API connectivity and polling cadence editable without leaving the dashboard."
                  submitLabel="Save runtime settings"
                  showGroupFields={false}
                  onSubmit={async (input) =>
                    commitDraft({
                      ...draftConfig,
                      apiBaseUrl: input.apiBaseUrl,
                      apiKey: input.apiKey,
                      pollingIntervalSeconds: input.pollingIntervalSeconds,
                    })
                  }
                />
                <GroupEditor
                  group={selectedDraftGroup}
                  isSaving={isSaving}
                  onChange={replaceCurrentGroup}
                  onSave={async () => {
                    if (!draftConfig) {
                      return;
                    }

                    await commitDraft(draftConfig);
                  }}
                />
                <WindowPolicyForm
                  layoutPreset={draftConfig.layoutPreset ?? "table"}
                  density={draftConfig.density ?? "comfortable"}
                  notificationsEnabled={draftConfig.notificationsEnabled ?? true}
                  windowPolicy={draftConfig.windowPolicy}
                  isSaving={isSaving}
                  onChange={(nextPolicy) =>
                    setDraftConfig((currentDraft) =>
                      currentDraft
                        ? {
                            ...currentDraft,
                            layoutPreset: nextPolicy.layoutPreset,
                            density: nextPolicy.density,
                            notificationsEnabled: nextPolicy.notificationsEnabled,
                            windowPolicy: nextPolicy.windowPolicy,
                          }
                        : currentDraft,
                    )
                  }
                  onSave={async () => {
                    if (!draftConfig) {
                      return;
                    }

                    await commitDraft(draftConfig);
                  }}
                />
                <PollingHealthPanel
                  diagnostics={snapshot.diagnostics}
                  health={snapshot.health}
                  runtimeStatus={getSnapshotRuntimeStatus(snapshot)}
                  onPollNow={pollNow}
                />
                <RecoveryPanel
                  items={recoveryItems}
                  onOpenInDashboard={(alertId) => {
                    const targetAlert = recoveryItems.find((item) => item.alert.id === alertId)?.alert;

                    if (targetAlert) {
                      void openAlertInDashboard(targetAlert);
                    }
                  }}
                  onMarkRead={(alertId) => {
                    const targetAlert = recoveryItems.find((item) => item.alert.id === alertId)?.alert;

                    if (targetAlert) {
                      void markAlertRead(targetAlert);
                    }
                  }}
                />
                <DiagnosticsPanel
                  diagnostics={snapshot.diagnostics}
                  issues={viewModel?.groupSnapshot.issues ?? []}
                  widgetRuntime={snapshot.widgetRuntime}
                />
              </div>
            }
          />
        ) : null}
      </div>
    </main>
  );
}
