import {
  createContext,
  useCallback,
  useEffect,
  useRef,
  useState,
  useSyncExternalStore,
} from "react";
import { BaseTheme, WbblWebappPreferencesStore } from "../../pkg/wbbl";

export const WbblPreferencesStoreContext =
  createContext<WbblWebappPreferencesStore>(WbblWebappPreferencesStore.empty());

export type ThemePreferences = {
  baseTheme: BaseTheme;
  currentTheme: BaseTheme;
  variables: { [key: string]: string };
};
export function useThemePreferences(
  store: WbblWebappPreferencesStore,
): ThemePreferences {
  let data = useRef<Omit<ThemePreferences, "currentTheme">>();
  let count = useRef<number>(0);
  let cacheHandle = useRef<number>(0);

  let subscribe = useCallback(
    (subscriber: () => void) => {
      if (count.current == 0) {
        cacheHandle.current = store.subscribe(() => {
          data.current = undefined;
        });
      }
      count.current = count.current + 1;
      let handle = store.subscribe(subscriber);
      return () => {
        count.current = count.current - 1;
        if (count.current === 0) {
          store.unsubscribe(cacheHandle.current);
        }
        store.unsubscribe(handle);
      };
    },
    [store],
  );

  let getSnapshot = useCallback(() => {
    if (data.current === undefined) {
      let baseTheme = store.get_base_theme();
      data.current = { baseTheme, variables: {} };
    }
    return data.current;
  }, [store]);

  const snapshot = useSyncExternalStore(subscribe, getSnapshot, getSnapshot);

  const [enrichedWithSystemThemeSnapshot, setEnrichedWithSystemThemeSnapshot] =
    useState({
      baseTheme: BaseTheme.System,
      currentTheme: BaseTheme.Light,
      variables: {},
    });

  useEffect(() => {
    if (snapshot.baseTheme === BaseTheme.System) {
      let query = window.matchMedia("(prefers-color-scheme: dark)");
      setEnrichedWithSystemThemeSnapshot({
        ...snapshot,
        currentTheme: query.matches ? BaseTheme.Dark : BaseTheme.Light,
      });
      if (query) {
        const listener = (ev: MediaQueryListEvent) => {
          setEnrichedWithSystemThemeSnapshot({
            ...snapshot,
            currentTheme: ev.matches ? BaseTheme.Dark : BaseTheme.Light,
          });
        };
        query.addEventListener("change", listener);
        return () => {
          query.removeEventListener("change", listener);
        };
      }
    } else {
      setEnrichedWithSystemThemeSnapshot({
        ...snapshot,
        currentTheme: snapshot.baseTheme,
      });
    }
  }, [snapshot, setEnrichedWithSystemThemeSnapshot]);

  return enrichedWithSystemThemeSnapshot;
}
