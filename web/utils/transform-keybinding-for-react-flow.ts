import { isMacLike } from "./is-mac-like";

export function transformKeybindingForReactFlow(keybinding: string) {
  let tokens = keybinding.split("+");
  if (isMacLike()) {
    tokens = tokens.map((x) => x.replace(/mod/g, "Meta"));
  } else {
    tokens = tokens.map((x) => x.replace(/mod/g, "Ctrl"));
  }
  tokens = tokens.map((x) => {
    if (x.length > 1) {
      return x[0].toUpperCase() + x.substring(1);
    }
    return x;
  });
  return tokens.join("+");
}
