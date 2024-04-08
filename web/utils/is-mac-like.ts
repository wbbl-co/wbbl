export function isMacLike(): boolean {
  if (
    (navigator as any).userAgentData &&
    (navigator as any).userAgentData.platform
  ) {
    return /macOS/i.test((navigator as any).userAgentData.platform);
  } else if (navigator.platform) {
    return /(Mac|iPhone|iPod|iPad)/i.test(navigator.platform);
  } else {
    return /mac/i.test(navigator.userAgent);
  }
}
