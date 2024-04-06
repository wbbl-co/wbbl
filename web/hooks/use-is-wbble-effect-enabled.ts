import { useViewport } from "@xyflow/react";

function getPrefersReducedMotion() {
  const mediaQueryList = window.matchMedia(
    "(prefers-reduced-motion: no-preference)",
  );

  const prefersReducedMotion = !mediaQueryList.matches;
  return prefersReducedMotion;
}

export default function useIsWbblEffectEnabled() {
  // TODO add additional check for visible node count
  const viewport = useViewport();
  return !getPrefersReducedMotion() && viewport.zoom > 0.5;
}
