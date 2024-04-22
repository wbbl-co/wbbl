let isMac: null | boolean = null;
export function isMacLike(): boolean {
  if (isMac !== null) {
    return !!isMac;
  }
  if (
    (navigator as any).userAgentData &&
    (navigator as any).userAgentData.platform
  ) {
    isMac = /macOS/i.test((navigator as any).userAgentData.platform);
  } else if (navigator.platform) {
    isMac = /(Mac|iPhone|iPod|iPad)/i.test(navigator.platform);
  } else {
    isMac = /mac/i.test(navigator.userAgent);
  }
  return isMac;
}
