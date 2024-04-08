import { isMacLike } from "./is-mac-like";

export default function normalizeKeybinding(binding: string) {
  if (isMacLike()) {
    return binding.replace(/meta/g, "mod");
  }

  return binding.replace(/ctrl/g, "mod");
}
