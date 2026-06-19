// Matches the layout shape sent by the Rust backend (src-tauri/src/keyboard.rs)
export interface KeyDef {
  code: string;
  label: string;
  col: number;
  row: number;
  colSpan?: number;
  rowSpan?: number;
}

export interface KeyboardLayout {
  main: KeyDef[];
  numpad: KeyDef[];
}
