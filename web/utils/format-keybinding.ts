import { isMacLike } from "./is-mac-like";

export default function formatKeybinding(binding: string) {
  if (isMacLike()) {
    binding = binding
      .replace(/mod/g, "⌘")
      .replace(/meta/g, "⌘")
      .replace(/shift/g, "⇧")
      .replace(/option/g, "⌥")
      .replace(/control/g, "⌃")
      .replace(/alt/g, "⌥");
  }

  return binding
    .replace(/mod/g, "ctrl")
    .replace(/backspace/g, "⌫")
    .replace(/delete/g, "DEL")
    .replace(/up/g, "↑")
    .replace(/down/g, "↓")
    .replace(/left/g, "←")
    .replace(/right/g, "→")
    .replace(/return/g, "⏎")
    .replace(/tab/g, "⇥")
    .replace(/home/g, "↖")
    .replace(/end/g, "↘")
    .replace(/minus/g, "-")
    .replace(/plus/g, "+")
    .replace(/(^space|(:?\+)space)/g, "␣")
    .split("+")
    .sort((a, b) => a.localeCompare(b))
    .join(" ")
    .toLocaleUpperCase();
}
