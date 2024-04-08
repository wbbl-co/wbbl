import { KeyboardShortcut } from "../pkg/wbbl";

const keybindingDescriptors: { [K in KeyboardShortcut]: string } = {
  [KeyboardShortcut.Copy]: "Copy",
  [KeyboardShortcut.Paste]: "Paste",
  [KeyboardShortcut.Cut]: "Cut",
  [KeyboardShortcut.Undo]: "Undo",
  [KeyboardShortcut.Redo]: "Redo",
  [KeyboardShortcut.QuickActions]: "Quick Actions",
  [KeyboardShortcut.OpenKeybindings]: "Toggle Keybindings",
  [KeyboardShortcut.Home]: "Home",
  [KeyboardShortcut.Delete]: "Delete",
  [KeyboardShortcut.Duplicate]: "Duplicate",
};

export default keybindingDescriptors;
