import {
  createContext,
  useCallback,
  useEffect,
  useRef,
  useState,
  useSyncExternalStore,
} from "react";
import {
  BaseTheme,
  KeyboardShortcut,
  WbblWebappNodeType,
  WbblWebappPreferencesStore,
} from "../../pkg/wbbl";

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
      currentTheme: window.matchMedia("(prefers-color-scheme: dark)").matches
        ? BaseTheme.Dark
        : BaseTheme.Light,
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

export type KeyboardPreferences = {
  keys: Map<KeyboardShortcut, string | null | undefined>;
  node_keys: Map<string, string | null | undefined>;
};

export function useKeyboardPreferences(
  store: WbblWebappPreferencesStore,
): KeyboardPreferences {
  let data = useRef<KeyboardPreferences>();
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
      let keys = store.get_keybindings() as Map<
        KeyboardShortcut,
        string | null | undefined
      >;
      let node_keys = store.get_node_keybindings() as Map<
        string,
        string | null | undefined
      >;
      data.current = { keys, node_keys };
    }
    return data.current;
  }, [store]);

  const snapshot = useSyncExternalStore(subscribe, getSnapshot, getSnapshot);

  return snapshot;
}

export function useFavouritesPreferences(
  store: WbblWebappPreferencesStore,
): WbblWebappNodeType[] {
  let data = useRef<WbblWebappNodeType[]>();
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
      let favourites = store.get_favourites();
      data.current = favourites;
    }
    return data.current;
  }, [store]);

  const snapshot = useSyncExternalStore(subscribe, getSnapshot, getSnapshot);

  return snapshot;
}

export function useIsFavouritePreference(
  store: WbblWebappPreferencesStore,
  type: WbblWebappNodeType,
  isOpen: boolean,
): boolean {
  let data = useRef<boolean | undefined>(undefined);
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

  useEffect(() => {
    data.current = store.is_favourite(type);
  }, [type, isOpen]);

  let getSnapshot = useCallback(() => {
    if (data.current === undefined) {
      let favourite = store.is_favourite(type);
      data.current = favourite;
    }
    return data.current;
  }, [store, type]);

  const snapshot = useSyncExternalStore(subscribe, getSnapshot, getSnapshot);

  return snapshot;
}

export function useKeyBinding(
  store: WbblWebappPreferencesStore,
  shortcut: KeyboardShortcut,
): string | undefined | null {
  let data = useRef<string | undefined | null>(undefined);
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

  useEffect(() => {
    data.current = store.get_keybinding(shortcut);
  }, [shortcut]);

  let getSnapshot = useCallback(() => {
    if (data.current === undefined) {
      data.current = store.get_keybinding(shortcut);
    }
    return data.current;
  }, [store, shortcut]);

  const snapshot = useSyncExternalStore(subscribe, getSnapshot, getSnapshot);

  return snapshot;
}

export function useNodeKeyBinding(
  store: WbblWebappPreferencesStore,
  node_type: WbblWebappNodeType,
): string | undefined | null {
  let data = useRef<string | undefined | null>(undefined);
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

  useEffect(() => {
    data.current = store.get_node_keybinding(node_type);
  }, [node_type]);

  let getSnapshot = useCallback(() => {
    if (data.current === undefined) {
      data.current = store.get_node_keybinding(node_type);
    }
    return data.current;
  }, [store, node_type]);

  const snapshot = useSyncExternalStore(subscribe, getSnapshot, getSnapshot);

  return snapshot;
}
