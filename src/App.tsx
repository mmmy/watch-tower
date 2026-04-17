import { useEffect, useMemo, useRef, useState } from "react";
import "./App.css";
import { Menu, MenuItem } from "@tauri-apps/api/menu";
import { PhysicalPosition, currentMonitor, getCurrentWindow, monitorFromPoint } from "@tauri-apps/api/window";
import {
  getRuntimeSnapshot,
  markSignalRead,
  quitApp,
  refreshSignals,
  setAlwaysOnTop,
  setEdgeMode,
  setEdgeWidth,
  setNotifications,
  setSound,
  subscribeRuntime,
  toggleMain,
} from "./runtime";
import type { RuntimeSignal, RuntimeSnapshot, SignalMutationInput, WatchGroup } from "./types";

type ViewMode = "main" | "widget";
type WidgetDockSide = "free" | "left" | "right" | "top" | "bottom";
type WidgetPlacement = {
  x: number;
  y: number;
  dock: WidgetDockSide;
  autoHidden: boolean;
};

const CELL_COUNT = 60;
const WIDGET_STORAGE_KEY = "signal-desk.widget-placement";
const WIDGET_EDGE_THRESHOLD = 56;
const WIDGET_PEEK = 24;
const WIDGET_VISIBLE_MARGIN = 12;
const WIDGET_AUTO_HIDE_DELAY = 520;
const WIDGET_DRAG_THRESHOLD = 8;

function detectViewMode(): ViewMode {
  const params = new URLSearchParams(window.location.search);
  return params.get("view") === "widget" ? "widget" : "main";
}

function periodToMs(period: string): number {
  if (period === "W") return 7 * 24 * 60 * 60 * 1000;
  if (period === "D") return 24 * 60 * 60 * 1000;
  if (period.endsWith("D")) {
    return Number.parseInt(period.slice(0, -1), 10) * 24 * 60 * 60 * 1000;
  }
  return Number.parseInt(period, 10) * 60 * 1000;
}

function buildTimeline(signal: RuntimeSignal, now: number) {
  const cells = new Array(CELL_COUNT).fill(0);
  const elapsed = Math.max(now - signal.trigger_time, 0);
  const candlesAgo = Math.floor(elapsed / periodToMs(signal.period));
  const activeIndex = CELL_COUNT - 1 - candlesAgo;

  if (activeIndex >= 0 && activeIndex < CELL_COUNT) {
    cells[activeIndex] = signal.side;
  }

  return cells;
}

function formatTime(ts: number) {
  return new Intl.DateTimeFormat("zh-CN", {
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  }).format(ts);
}

function getConnectionState(lastUpdatedAt: number, intervalSecs: number, now: number) {
  const elapsedMs = Math.max(0, now - lastUpdatedAt);
  const expectedMs = Math.max(intervalSecs, 1) * 1000;

  if (elapsedMs <= expectedMs * 2) {
    return { label: "连接正常", tone: "online" as const };
  }

  if (elapsedMs <= expectedMs * 4) {
    return { label: "连接延迟", tone: "lagging" as const };
  }

  return { label: "连接超时", tone: "offline" as const };
}

function signalKey(signal: RuntimeSignal): SignalMutationInput {
  return {
    group_id: signal.group_id,
    signal_type: signal.signal_type,
    period: signal.period,
  };
}

function updateSnapshot(
  snapshot: RuntimeSnapshot,
  input: SignalMutationInput,
  mutate: (signal: RuntimeSignal) => RuntimeSignal,
): RuntimeSnapshot {
  const signals = snapshot.signals.map((signal) => {
    if (
      signal.group_id === input.group_id &&
      signal.signal_type === input.signal_type &&
      signal.period === input.period
    ) {
      return mutate(signal);
    }
    return signal;
  });

  const unreadCount = signals.filter((signal) => signal.unread && !signal.deleted).length;
  return { ...snapshot, signals, unread_count: unreadCount };
}

function groupedSignals(snapshot: RuntimeSnapshot) {
  const visibleSignals = snapshot.signals.filter((signal) => !signal.deleted);

  return snapshot.config.groups
    .filter((group) => group.enabled)
    .map((group) => ({
      group,
      signals: visibleSignals.filter((signal) => signal.group_id === group.id),
    }));
}

function clamp(value: number, min: number, max: number) {
  return Math.min(Math.max(value, min), max);
}

function saveWidgetPlacement(placement: WidgetPlacement) {
  localStorage.setItem(WIDGET_STORAGE_KEY, JSON.stringify(placement));
}

function loadWidgetPlacement(): WidgetPlacement | null {
  const raw = localStorage.getItem(WIDGET_STORAGE_KEY);
  if (!raw) {
    return null;
  }

  try {
    return JSON.parse(raw) as WidgetPlacement;
  } catch (error) {
    console.error("Failed to parse widget placement", error);
    return null;
  }
}

function buildWidgetPlacement(
  x: number,
  y: number,
  width: number,
  height: number,
  monitor: Awaited<ReturnType<typeof currentMonitor>>,
): WidgetPlacement {
  if (!monitor) {
    return { x, y, dock: "free", autoHidden: false };
  }

  const minX = monitor.workArea.position.x;
  const minY = monitor.workArea.position.y;
  const maxX = minX + monitor.workArea.size.width - width;
  const maxY = minY + monitor.workArea.size.height - height;

  const clampedX = clamp(x, minX, maxX);
  const clampedY = clamp(y, minY, maxY);

  const leftDistance = Math.abs(clampedX - minX);
  const rightDistance = Math.abs(maxX - clampedX);
  const topDistance = Math.abs(clampedY - minY);
  const bottomDistance = Math.abs(maxY - clampedY);

  const nearest = [
    { dock: "left" as const, distance: leftDistance },
    { dock: "right" as const, distance: rightDistance },
    { dock: "top" as const, distance: topDistance },
    { dock: "bottom" as const, distance: bottomDistance },
  ].sort((a, b) => a.distance - b.distance)[0];

  if (nearest.distance > WIDGET_EDGE_THRESHOLD) {
    return { x: clampedX, y: clampedY, dock: "free", autoHidden: false };
  }

  if (nearest.dock === "left") {
    return {
      x: minX - width + WIDGET_PEEK,
      y: clampedY,
      dock: "left",
      autoHidden: true,
    };
  }

  if (nearest.dock === "right") {
    return {
      x: minX + monitor.workArea.size.width - WIDGET_PEEK,
      y: clampedY,
      dock: "right",
      autoHidden: true,
    };
  }

  if (nearest.dock === "top") {
    return {
      x: clampedX,
      y: minY,
      dock: "top",
      autoHidden: false,
    };
  }

  return {
    x: clampedX,
    y: maxY,
    dock: "bottom",
    autoHidden: false,
  };
}

function revealWidgetPlacement(
  placement: WidgetPlacement,
  width: number,
  monitor: Awaited<ReturnType<typeof currentMonitor>>,
): WidgetPlacement {
  if (!monitor || !placement.autoHidden) {
    return placement;
  }

  const minX = monitor.workArea.position.x;
  const maxX = minX + monitor.workArea.size.width - width;

  if (placement.dock === "left") {
    return { ...placement, x: minX + WIDGET_VISIBLE_MARGIN, autoHidden: false };
  }

  if (placement.dock === "right") {
    return { ...placement, x: maxX - WIDGET_VISIBLE_MARGIN, autoHidden: false };
  }

  return placement;
}

function hideWidgetPlacement(
  placement: WidgetPlacement,
  width: number,
  monitor: Awaited<ReturnType<typeof currentMonitor>>,
): WidgetPlacement {
  if (!monitor || (placement.dock !== "left" && placement.dock !== "right")) {
    return placement;
  }

  const minX = monitor.workArea.position.x;
  const maxX = minX + monitor.workArea.size.width - width;

  if (placement.dock === "left") {
    return { ...placement, x: minX - width + WIDGET_PEEK, autoHidden: true };
  }

  return { ...placement, x: maxX + width - WIDGET_PEEK, autoHidden: true };
}

function WidgetView({ snapshot }: { snapshot: RuntimeSnapshot | null }) {
  const hideTimerRef = useRef<number | null>(null);
  const draggingWindowRef = useRef(false);
  const currentPlacementRef = useRef<WidgetPlacement | null>(null);
  const animationFrameRef = useRef<number | null>(null);
  const animationTokenRef = useRef(0);
  const moveDebounceRef = useRef<number | null>(null);
  const programmaticMoveRef = useRef(false);
  const pointerGestureRef = useRef<{
    pointerId: number;
    startClientX: number;
    startClientY: number;
    dragTriggered: boolean;
  } | null>(null);

  useEffect(() => {
    document.documentElement.dataset.view = "widget";
    document.body.dataset.view = "widget";

    void restoreWidgetPlacement();
    const appWindow = getCurrentWindow();
    let unlistenMoved: (() => void) | undefined;

    void appWindow.onMoved(() => {
      if (programmaticMoveRef.current) {
        return;
      }

      draggingWindowRef.current = true;
      cancelHideTimer();

      if (moveDebounceRef.current) {
        window.clearTimeout(moveDebounceRef.current);
      }

      moveDebounceRef.current = window.setTimeout(() => {
        moveDebounceRef.current = null;
        draggingWindowRef.current = false;
        void snapAndPersistWidgetPosition();
      }, 140);
    }).then((unlisten) => {
      unlistenMoved = unlisten;
    });

    return () => {
      if (hideTimerRef.current) {
        window.clearTimeout(hideTimerRef.current);
      }
      if (moveDebounceRef.current) {
        window.clearTimeout(moveDebounceRef.current);
      }
      if (animationFrameRef.current) {
        window.cancelAnimationFrame(animationFrameRef.current);
      }
      unlistenMoved?.();
      delete document.documentElement.dataset.view;
      delete document.body.dataset.view;
    };
  }, []);

  useEffect(() => {
    function clearGesture() {
      pointerGestureRef.current = null;
    }

    window.addEventListener("pointerup", clearGesture);
    window.addEventListener("pointercancel", clearGesture);

    return () => {
      window.removeEventListener("pointerup", clearGesture);
      window.removeEventListener("pointercancel", clearGesture);
    };
  }, []);

  async function resolveWidgetGeometry() {
    const appWindow = getCurrentWindow();
    const [position, size] = await Promise.all([appWindow.outerPosition(), appWindow.outerSize()]);
    const monitor =
      (await monitorFromPoint(position.x + Math.floor(size.width / 2), position.y + Math.floor(size.height / 2))) ??
      (await currentMonitor());

    return { appWindow, position, size, monitor };
  }

  async function animateWindowToPosition(targetX: number, targetY: number, duration = 180) {
    const appWindow = getCurrentWindow();
    const startPosition = await appWindow.outerPosition();

    if (startPosition.x === targetX && startPosition.y === targetY) {
      await appWindow.setPosition(new PhysicalPosition(targetX, targetY));
      return;
    }

    const animationId = animationTokenRef.current + 1;
    animationTokenRef.current = animationId;

    if (animationFrameRef.current) {
      window.cancelAnimationFrame(animationFrameRef.current);
      animationFrameRef.current = null;
    }

    await new Promise<void>((resolve) => {
      const startTime = performance.now();
      const deltaX = targetX - startPosition.x;
      const deltaY = targetY - startPosition.y;

      const step = (now: number) => {
        if (animationTokenRef.current !== animationId) {
          resolve();
          return;
        }

        const progress = Math.min((now - startTime) / duration, 1);
        const eased = 1 - (1 - progress) ** 3;
        const nextX = Math.round(startPosition.x + deltaX * eased);
        const nextY = Math.round(startPosition.y + deltaY * eased);
        void appWindow.setPosition(new PhysicalPosition(nextX, nextY));

        if (progress < 1) {
          animationFrameRef.current = window.requestAnimationFrame(step);
          return;
        }

        animationFrameRef.current = null;
        resolve();
      };

      animationFrameRef.current = window.requestAnimationFrame(step);
    });
  }

  async function applyPlacement(placement: WidgetPlacement, options?: { animate?: boolean }) {
    programmaticMoveRef.current = true;
    try {
      if (options?.animate) {
        await animateWindowToPosition(placement.x, placement.y);
      } else {
        const appWindow = getCurrentWindow();
        await appWindow.setPosition(new PhysicalPosition(placement.x, placement.y));
      }
    } finally {
      window.setTimeout(() => {
        programmaticMoveRef.current = false;
      }, 0);
    }

    currentPlacementRef.current = placement;
    saveWidgetPlacement(placement);
  }

  async function restoreWidgetPlacement() {
    const saved = loadWidgetPlacement();
    if (!saved) {
      return;
    }

    try {
      const { size, monitor } = await resolveWidgetGeometry();
      const width = size.width;
      const height = size.height;

      let placement = buildWidgetPlacement(saved.x, saved.y, width, height, monitor);
      if (saved.dock === "left" || saved.dock === "right") {
        placement = {
          ...placement,
          dock: saved.dock,
          autoHidden: saved.autoHidden,
        };
        if (saved.autoHidden) {
          placement = hideWidgetPlacement(placement, width, monitor);
        }
      }

      await applyPlacement(placement);
    } catch (error) {
      console.error("Failed to restore widget placement", error);
    }
  }

  async function snapAndPersistWidgetPosition() {
    try {
      const { position, size, monitor } = await resolveWidgetGeometry();
      const placement = buildWidgetPlacement(position.x, position.y, size.width, size.height, monitor);
      await applyPlacement(placement, { animate: true });
    } catch (error) {
      console.error("Failed to snap widget position", error);
    }
  }

  async function revealDockedWidget() {
    if (draggingWindowRef.current || !currentPlacementRef.current?.autoHidden) {
      return;
    }

    try {
      const { size, monitor } = await resolveWidgetGeometry();
      const revealed = revealWidgetPlacement(currentPlacementRef.current, size.width, monitor);
      await applyPlacement(revealed, { animate: true });
    } catch (error) {
      console.error("Failed to reveal widget", error);
    }
  }

  async function hideDockedWidget() {
    if (draggingWindowRef.current || !currentPlacementRef.current) {
      return;
    }

    try {
      const { size, monitor } = await resolveWidgetGeometry();
      const hidden = hideWidgetPlacement(currentPlacementRef.current, size.width, monitor);
      await applyPlacement(hidden, { animate: true });
    } catch (error) {
      console.error("Failed to auto-hide widget", error);
    }
  }

  function cancelHideTimer() {
    if (hideTimerRef.current) {
      window.clearTimeout(hideTimerRef.current);
      hideTimerRef.current = null;
    }
  }

  async function handleWidgetPointerDown(event: React.PointerEvent<HTMLDivElement>) {
    if (event.button !== 0 || draggingWindowRef.current || pointerGestureRef.current) {
      return;
    }

    cancelHideTimer();
    pointerGestureRef.current = {
      pointerId: event.pointerId,
      startClientX: event.clientX,
      startClientY: event.clientY,
      dragTriggered: false,
    };
    event.currentTarget.setPointerCapture(event.pointerId);
  }

  async function handleWidgetPointerMove(event: React.PointerEvent<HTMLDivElement>) {
    const gesture = pointerGestureRef.current;
    if (!gesture || gesture.pointerId !== event.pointerId || gesture.dragTriggered) {
      return;
    }

    const deltaX = event.clientX - gesture.startClientX;
    const deltaY = event.clientY - gesture.startClientY;
    const distance = Math.hypot(deltaX, deltaY);

    if (distance < WIDGET_DRAG_THRESHOLD) {
      return;
    }

    gesture.dragTriggered = true;
    draggingWindowRef.current = true;
    cancelHideTimer();

    try {
      await getCurrentWindow().startDragging();
    } catch (error) {
      console.error("startDragging failed", error);
      draggingWindowRef.current = false;
      pointerGestureRef.current = null;
    }
  }

  async function finishWidgetPointer(event: React.PointerEvent<HTMLDivElement>) {
    const gesture = pointerGestureRef.current;
    if (!gesture || gesture.pointerId !== event.pointerId) {
      return;
    }

    pointerGestureRef.current = null;
    if (event.currentTarget.hasPointerCapture(event.pointerId)) {
      event.currentTarget.releasePointerCapture(event.pointerId);
    }

    if (!gesture.dragTriggered) {
      await toggleMain();
    }
  }

  function handlePointerEnter() {
    cancelHideTimer();
    void revealDockedWidget();
  }

  function handlePointerLeave() {
    if (draggingWindowRef.current || pointerGestureRef.current) {
      return;
    }

    cancelHideTimer();
    hideTimerRef.current = window.setTimeout(() => {
      void hideDockedWidget();
      hideTimerRef.current = null;
    }, WIDGET_AUTO_HIDE_DELAY);
  }

  async function handleContextMenu(event: React.MouseEvent<HTMLDivElement>) {
    event.preventDefault();
    cancelHideTimer();
    await revealDockedWidget();

    const items = await Promise.all([
      MenuItem.new({
        id: "toggle-main",
        text: "打开主窗",
        action: () => {
          void toggleMain();
        },
      }),
      MenuItem.new({
        id: "refresh-now",
        text: "立即刷新",
        action: () => {
          void refreshSignals();
        },
      }),
      MenuItem.new({
        id: "toggle-pin",
        text: snapshot?.always_on_top ? "取消置顶" : "窗口置顶",
        action: () => {
          if (!snapshot) {
            return;
          }
          void setAlwaysOnTop(!snapshot.always_on_top);
        },
      }),
      MenuItem.new({
        id: "quit-app",
        text: "退出应用",
        action: () => {
          void quitApp();
        },
      }),
    ]);

    const menu = await Menu.new({ items });
    await menu.popup(undefined, getCurrentWindow());
  }

  return (
    <div className="widget-shell" onContextMenu={(event) => event.preventDefault()}>
      <div
        className={`widget-orb ${snapshot && snapshot.unread_count > 0 ? "is-hot" : "is-calm"}`}
        onContextMenu={(event) => void handleContextMenu(event)}
        onPointerEnter={handlePointerEnter}
        onPointerLeave={handlePointerLeave}
      >
        <span className="widget-glow" />
        <div
          className="widget-drag-hit-area"
          onPointerCancel={(event) => void finishWidgetPointer(event)}
          onPointerDown={(event) => void handleWidgetPointerDown(event)}
          onPointerMove={(event) => void handleWidgetPointerMove(event)}
          onPointerUp={(event) => void finishWidgetPointer(event)}
        >
          <div className="widget-drag-layer" aria-hidden="true">
            <span className="widget-count">{snapshot?.unread_count ?? 0}</span>
          </div>
        </div>
      </div>
    </div>
  );
}

function ToolbarToggle({
  checked,
  disabled,
  label,
  onChange,
}: {
  checked: boolean;
  disabled?: boolean;
  label: string;
  onChange: (checked: boolean) => void;
}) {
  return (
    <label className="toolbar-toggle">
      <input
        checked={checked}
        disabled={disabled}
        onChange={(event) => onChange(event.currentTarget.checked)}
        type="checkbox"
      />
      <span>{label}</span>
    </label>
  );
}

function PeriodRow({
  group,
  signal,
  onMarkRead,
}: {
  group: WatchGroup;
  signal: RuntimeSignal;
  onMarkRead: (signal: RuntimeSignal, read: boolean) => void;
}) {
  const now = Date.now();
  const cells = buildTimeline(signal, now);
  const isLong = signal.side > 0;

  return (
    <div className={`period-row ${signal.unread ? "is-unread" : ""}`}>
      <button
        className={`period-label ${signal.unread ? "unread" : "read"} ${isLong ? "long" : "short"}`}
        onClick={() => onMarkRead(signal, signal.unread)}
        title={signal.unread ? "标记已读" : "恢复未读"}
        type="button"
      >
        {signal.unread ? "•" : ""}
        {signal.period}
      </button>
      <div className="period-track" aria-label={`${group.symbol} ${signal.period} ${signal.signal_type}`}>
        {cells.map((cell, index) => (
          <span
            // eslint-disable-next-line react/no-array-index-key
            key={`${signal.group_id}-${signal.signal_type}-${signal.period}-${index}`}
            className={[
              "period-cell",
              cell === 1 ? "is-long" : "",
              cell === -1 ? "is-short" : "",
              signal.unread && cell !== 0 ? "is-unread" : "",
            ]
              .filter(Boolean)
              .join(" ")}
          />
        ))}
      </div>
    </div>
  );
}

function GroupPanel({
  group,
  signals,
  onMarkRead,
  onMarkGroupRead,
}: {
  group: WatchGroup;
  signals: RuntimeSignal[];
  onMarkRead: (signal: RuntimeSignal, read: boolean) => void;
  onMarkGroupRead: (signals: RuntimeSignal[]) => void;
}) {
  const unreadCount = signals.filter((signal) => signal.unread).length;
  const bySignalType = group.signal_types.map((signalType) => ({
    signalType,
    items: group.periods
      .map((period) =>
        signals.find((signal) => signal.signal_type === signalType && signal.period === period),
      )
      .filter((signal): signal is RuntimeSignal => Boolean(signal)),
  }));

  return (
    <section className="group-card">
      <header className="group-topline">
        <div className="group-title-wrap">
          <strong className="group-symbol">{group.symbol}</strong>
          <span className="group-name">{group.name}</span>
        </div>
        <div className="group-actions">
          {unreadCount > 0 ? <span className="group-unread">{unreadCount} unread</span> : null}
          <button
            className="group-read-button"
            onClick={() => onMarkGroupRead(signals.filter((signal) => signal.unread))}
            type="button"
          >
            全部已读
          </button>
        </div>
      </header>
      {bySignalType.map(({ signalType, items }) => (
        <div className="signal-block" key={`${group.id}-${signalType}`}>
          <div className="signal-block-header">
            <span>{signalType}</span>
            <span>{items.length} rows</span>
          </div>
          <div className="period-list">
            {items.map((signal) => (
              <PeriodRow
                key={`${signal.group_id}-${signal.signal_type}-${signal.period}`}
                group={group}
                signal={signal}
                onMarkRead={onMarkRead}
              />
            ))}
          </div>
        </div>
      ))}
    </section>
  );
}

function MainView({ snapshot, setSnapshot }: { snapshot: RuntimeSnapshot; setSnapshot: (value: RuntimeSnapshot) => void }) {
  const groups = useMemo(() => groupedSignals(snapshot), [snapshot]);
  const [edgeWidthInput, setEdgeWidthInput] = useState(() => String(Math.round(snapshot.config.ui.edge_width)));
  const [now, setNow] = useState(() => Date.now());
  const [moreOpen, setMoreOpen] = useState(false);

  useEffect(() => {
    document.documentElement.dataset.view = "main";
    document.body.dataset.view = "main";
    return () => {
      delete document.documentElement.dataset.view;
      delete document.body.dataset.view;
    };
  }, []);

  useEffect(() => {
    const timer = window.setInterval(() => {
      setNow(Date.now());
    }, 1000);

    return () => window.clearInterval(timer);
  }, []);

  useEffect(() => {
    if (!moreOpen) {
      return;
    }

    function handlePointerDown() {
      setMoreOpen(false);
    }

    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") {
        setMoreOpen(false);
      }
    }

    window.addEventListener("pointerdown", handlePointerDown);
    window.addEventListener("keydown", handleKeyDown);

    return () => {
      window.removeEventListener("pointerdown", handlePointerDown);
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [moreOpen]);

  useEffect(() => {
    setEdgeWidthInput(String(Math.round(snapshot.config.ui.edge_width)));
  }, [snapshot.config.ui.edge_width]);

  async function performOptimisticUpdate(
    mutate: (value: RuntimeSnapshot) => RuntimeSnapshot,
    request: () => Promise<RuntimeSnapshot>,
  ) {
    const previous = snapshot;
    setSnapshot(mutate(previous));
    try {
      setSnapshot(await request());
    } catch (error) {
      console.error(error);
      setSnapshot(previous);
    }
  }

  function onMarkRead(signal: RuntimeSignal, read: boolean) {
    const input = signalKey(signal);
    void performOptimisticUpdate(
      (value) =>
        updateSnapshot(value, input, (current) => ({
          ...current,
          unread: !read,
        })),
      () => markSignalRead(input, read),
    );
  }

  function onMarkGroupRead(signals: RuntimeSignal[]) {
    if (signals.length === 0) {
      return;
    }

    const previous = snapshot;
    const inputs = signals.map(signalKey);
    setSnapshot({
      ...previous,
      signals: previous.signals.map((signal) =>
        inputs.some(
          (input) =>
            input.group_id === signal.group_id &&
            input.signal_type === signal.signal_type &&
            input.period === signal.period,
        )
          ? { ...signal, unread: false }
          : signal,
      ),
      unread_count: Math.max(0, previous.unread_count - signals.length),
    });

    void Promise.all(signals.map((signal) => markSignalRead(signalKey(signal), true)))
      .then((results) => {
        const latest = results.length > 0 ? results[results.length - 1] : undefined;
        if (latest) {
          setSnapshot(latest);
        }
      })
      .catch((error) => {
        console.error(error);
        setSnapshot(previous);
      });
  }

  function submitEdgeWidth(nextValue: string) {
    const parsed = Number.parseFloat(nextValue);
    if (Number.isNaN(parsed)) {
      setEdgeWidthInput(String(Math.round(snapshot.config.ui.edge_width)));
      return;
    }

    const normalized = Math.min(Math.max(parsed, 160), 480);
    setEdgeWidthInput(String(Math.round(normalized)));
    void setEdgeWidth(normalized).then(setSnapshot);
  }

  const connectionState = getConnectionState(
    snapshot.last_updated_at,
    snapshot.config.poll.interval_secs,
    now,
  );

  return (
    <div className="plugin-shell">
      <header className="plugin-toolbar">
        <div className="toolbar-primary">
          <ToolbarToggle
            checked={snapshot.always_on_top}
            label="置顶"
            onChange={(value) => void setAlwaysOnTop(value).then(setSnapshot)}
          />
          <ToolbarToggle
            checked={snapshot.edge_mode}
            label="贴边模式"
            onChange={(value) => void setEdgeMode(value).then(setSnapshot)}
          />
          <ToolbarToggle
            checked={snapshot.config.ui.notifications}
            label="通知"
            onChange={(value) => void setNotifications(value).then(setSnapshot)}
          />
          <ToolbarToggle
            checked={snapshot.config.ui.sound}
            label="声音"
            onChange={(value) => void setSound(value).then(setSnapshot)}
          />
        </div>
        <div className="toolbar-more">
          <button
            onClick={(event) => {
              event.stopPropagation();
              setMoreOpen((value) => !value);
            }}
            type="button"
          >
            更多
          </button>
          {moreOpen ? (
            <div
              className="toolbar-menu"
              onPointerDown={(event) => event.stopPropagation()}
              role="menu"
            >
              <button onClick={() => void refreshSignals().then(setSnapshot).finally(() => setMoreOpen(false))} type="button">
                立即轮询
              </button>
              <label className="toolbar-menu-field">
                <span>贴边宽度</span>
                <input
                  className="toolbar-number"
                  inputMode="numeric"
                  onBlur={(event) => submitEdgeWidth(event.currentTarget.value)}
                  onChange={(event) => setEdgeWidthInput(event.currentTarget.value)}
                  onKeyDown={(event) => {
                    if (event.key === "Enter") {
                      submitEdgeWidth((event.target as HTMLInputElement).value);
                    }
                  }}
                  type="number"
                  value={edgeWidthInput}
                />
              </label>
              <button disabled type="button">
                保存配置
              </button>
              <button onClick={() => void toggleMain().finally(() => setMoreOpen(false))} type="button">
                隐藏
              </button>
              <button onClick={() => void quitApp()} type="button">
                退出
              </button>
            </div>
          ) : null}
        </div>
      </header>

      <div className="plugin-statusbar">
        <span className={`status-highlight ${snapshot.unread_count === 0 ? "is-clear" : ""}`}>
          Total unread:{snapshot.unread_count}
        </span>
        <span>last poll: {formatTime(snapshot.last_updated_at)}</span>
        <span className={`connection-status ${connectionState.tone}`}>
          <span className="connection-dot" />
          {connectionState.label}
        </span>
      </div>

      <main className="plugin-content">
        {groups.map(({ group, signals }) => (
          <GroupPanel
            key={group.id}
            group={group}
            signals={signals}
            onMarkGroupRead={onMarkGroupRead}
            onMarkRead={onMarkRead}
          />
        ))}
      </main>
    </div>
  );
}

function App() {
  const [snapshot, setSnapshot] = useState<RuntimeSnapshot | null>(null);
  const viewMode = useMemo(detectViewMode, []);

  useEffect(() => {
    let disposed = false;
    let cleanup: (() => void) | undefined;

    void getRuntimeSnapshot().then((value) => {
      if (!disposed) {
        setSnapshot(value);
      }
    });

    void subscribeRuntime((value) => {
      if (!disposed) {
        setSnapshot(value);
      }
    }).then((unlisten) => {
      cleanup = unlisten;
    });

    return () => {
      disposed = true;
      cleanup?.();
    };
  }, []);

  if (viewMode === "widget") {
    return <WidgetView snapshot={snapshot} />;
  }

  if (!snapshot) {
    return <div className="loading-state">正在加载运行时骨架...</div>;
  }

  return <MainView snapshot={snapshot} setSnapshot={setSnapshot} />;
}

export default App;
