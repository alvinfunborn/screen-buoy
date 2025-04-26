export interface HintType {
  z_index: number;
  style: string;
  element_control_types: number[];
}

export interface HintConfig {
  charsets: string[][];
  charset_extra: string[];
  style: string;
  types: Record<string, HintType>;
  grid: GridConfig;
}

export interface GridConfig {
  rows: number;
  columns: number;
  show_at_rows: number[];
  show_at_columns: number[];
  hint_type: string;
}

export interface LeftRightConfig {
  left?: string | null;
  right?: string | null;
}

export interface KeyboardConfig {
  available_key: Record<string, number>;
  propagation_modifier: string[];
  map_left_right: Record<string, LeftRightConfig>;
}

export interface MouseStep {
  x: number;
  y: number;
  modifier: string[];
}

export interface MouseStepConfig {
  translate: MouseStep[];
  scroll: MouseStep[];
  drag: MouseStep[];
}

export interface MouseConfig {
  step: MouseStepConfig;
}

export interface DirectionKeybindingsConfig {
  up: string[];
  down: string[];
  left: string[];
  right: string[];
}

export interface GlobalKeybindingConfig {
  move_to_hint: string[];
  move_to_hint_exit: string[];
  left_click_exit: string[];
  hold_at_hint: string[];
  exit: string[];
  translate: DirectionKeybindingsConfig;
}

export interface AtHintKeybindingConfig {
  exit: string[];
  left_click: string[];
  left_click_exit: string[];
  double_click: string[];
  double_click_exit: string[];
  right_click: string[];
  right_click_exit: string[];
  middle_click: string[];
  middle_click_exit: string[];
  translate: DirectionKeybindingsConfig;
  drag: DirectionKeybindingsConfig;
  scroll: DirectionKeybindingsConfig;
}

export interface KeybindingConfig {
  global: GlobalKeybindingConfig;
  at_hint: AtHintKeybindingConfig;
  hotkey_buoy: string;
}

export interface SystemConfig {
  start_in_tray: boolean;
  show_tray_icon: boolean;
  start_at_login: boolean;
}

export interface UiAutomationConfig {
  collect_interval: number;
  cache_ttl: number;
}

export interface Config {
  hint: HintConfig;
  keybinding: KeybindingConfig;
  mouse: MouseConfig;
  keyboard: KeyboardConfig;
  system: SystemConfig;
  ui_automation: UiAutomationConfig;
} 