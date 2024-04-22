import { useViewport } from "@xyflow/react";
import { useContext, useEffect, useState } from "react";
import {
  WbblPreferencesStoreContext,
  useIsWobbleEffectEnabledInPreferences,
} from "./use-preferences-store";

function usePrefersReducedMotion() {
  const [prefersReducedMotion, setPrefersReducedMotion] = useState(false);
  useEffect(() => {
    const mediaQueryList = window.matchMedia(
      "(prefers-reduced-motion: no-preference)",
    );
    setPrefersReducedMotion(!mediaQueryList.matches);
    mediaQueryList.addEventListener("change", (evt) => {
      setPrefersReducedMotion(!evt.matches);
    });
  }, []);

  return prefersReducedMotion;
}

export default function useIsWbblEffectEnabled() {
  // TODO add additional check for visible node count
  const viewport = useViewport();
  const preferencesStore = useContext(WbblPreferencesStoreContext);
  const isWbblEnabledInPreferences =
    useIsWobbleEffectEnabledInPreferences(preferencesStore);
  const prefersReducedMotion = usePrefersReducedMotion();
  return (
    isWbblEnabledInPreferences && !prefersReducedMotion && viewport.zoom > 0.5
  );
}
