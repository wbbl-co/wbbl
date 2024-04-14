import { KeyboardShortcut } from "../pkg/wbbl";

const keybindingDescriptors: { [K in KeyboardShortcut]: string } = {
  [KeyboardShortcut.Copy]: "Copy",
  [KeyboardShortcut.Paste]: "Paste",
  [KeyboardShortcut.Cut]: "Cut",
  [KeyboardShortcut.Undo]: "Undo",
  [KeyboardShortcut.Redo]: "Redo",
  [KeyboardShortcut.QuickActions]: "Quick Actions",
  [KeyboardShortcut.OpenKeybindings]: "Open Keybindings",
  [KeyboardShortcut.Home]: "Home",
  [KeyboardShortcut.Delete]: "Delete",
  [KeyboardShortcut.Duplicate]: "Duplicate",
  [KeyboardShortcut.Help]: "Help",
  [KeyboardShortcut.LinkToPreview]: "Link to Preview",
  [KeyboardShortcut.Selection]: "Selection",
  [KeyboardShortcut.SelectAll]: "Select All",
  [KeyboardShortcut.SelectNone]: "Select None",
  [KeyboardShortcut.AutoLayout]: "Automatically Reposition Nodes",
  [KeyboardShortcut.GroupNodes]: "Group Nodes",
  [KeyboardShortcut.UngroupNodes]: "Ungroup Nodes",
};

export default keybindingDescriptors;
