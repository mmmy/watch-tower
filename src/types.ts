export type ApiConfig = {
  base_url: string;
  api_key: string;
};

export type PollConfig = {
  interval_secs: number;
  page_size: number;
};

export type UiConfig = {
  edge_mode: boolean;
  edge_width: number;
  always_on_top: boolean;
  notifications: boolean;
  sound: boolean;
};

export type WatchGroup = {
  id: string;
  name: string;
  symbol: string;
  periods: string[];
  signal_types: string[];
  enabled: boolean;
};

export type AppConfig = {
  api: ApiConfig;
  poll: PollConfig;
  ui: UiConfig;
  groups: WatchGroup[];
};

export type RuntimeSignal = {
  group_id: string;
  group_name: string;
  symbol: string;
  period: string;
  signal_type: string;
  side: 1 | -1;
  trigger_time: number;
  unread: boolean;
  deleted: boolean;
};

export type RuntimeSnapshot = {
  config: AppConfig;
  signals: RuntimeSignal[];
  unread_count: number;
  last_tick: number;
  last_updated_at: number;
  always_on_top: boolean;
  edge_mode: boolean;
  main_visible: boolean;
};

export type SignalMutationInput = {
  group_id: string;
  signal_type: string;
  period: string;
};
