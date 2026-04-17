import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { RuntimeSnapshot, SignalMutationInput } from "./types";

export const RUNTIME_EVENT = "runtime://state-changed";

export function getRuntimeSnapshot() {
  return invoke<RuntimeSnapshot>("get_runtime_snapshot");
}

export function markSignalRead(input: SignalMutationInput, read: boolean) {
  return invoke<RuntimeSnapshot>("mark_signal_read", { input, read });
}

export function refreshSignals() {
  return invoke<RuntimeSnapshot>("refresh_signals");
}

export function toggleMain() {
  return invoke<void>("toggle_main");
}

export function setAlwaysOnTop(pinned: boolean) {
  return invoke<RuntimeSnapshot>("set_always_on_top", { pinned });
}

export function setEdgeMode(enabled: boolean) {
  return invoke<RuntimeSnapshot>("set_edge_mode", { enabled });
}

export function setEdgeWidth(width: number) {
  return invoke<RuntimeSnapshot>("set_edge_width", { width });
}

export function setNotifications(enabled: boolean) {
  return invoke<RuntimeSnapshot>("set_notifications", { enabled });
}

export function setSound(enabled: boolean) {
  return invoke<RuntimeSnapshot>("set_sound", { enabled });
}

export function quitApp() {
  return invoke<void>("quit_app");
}

export function subscribeRuntime(listener: (snapshot: RuntimeSnapshot) => void) {
  return listen<RuntimeSnapshot>(RUNTIME_EVENT, (event) => listener(event.payload));
}
