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
  EdgeStyle,
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
  const data = useRef<Omit<ThemePreferences, "currentTheme">>();
  const count = useRef<number>(0);
  const cacheHandle = useRef<number>(0);

  const subscribe = useCallback(
    (subscriber: () => void) => {
      if (count.current == 0) {
        cacheHandle.current = store.subscribe(() => {
          data.current = undefined;
        });
      }
      count.current = count.current + 1;
      const handle = store.subscribe(subscriber);
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

  const getSnapshot = useCallback(() => {
    if (data.current === undefined) {
      const baseTheme = store.get_base_theme();
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
      const query = window.matchMedia("(prefers-color-scheme: dark)");
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
  const data = useRef<KeyboardPreferences>();
  const count = useRef<number>(0);
  const cacheHandle = useRef<number>(0);

  const subscribe = useCallback(
    (subscriber: () => void) => {
      if (count.current == 0) {
        cacheHandle.current = store.subscribe(() => {
          data.current = undefined;
        });
      }
      count.current = count.current + 1;
      const handle = store.subscribe(subscriber);
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

  const getSnapshot = useCallback(() => {
    if (data.current === undefined) {
      const keys = store.get_keybindings() as Map<
        KeyboardShortcut,
        string | null | undefined
      >;
      const node_keys = store.get_node_keybindings() as Map<
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
  const data = useRef<WbblWebappNodeType[]>();
  const count = useRef<number>(0);
  const cacheHandle = useRef<number>(0);

  const subscribe = useCallback(
    (subscriber: () => void) => {
      if (count.current == 0) {
        cacheHandle.current = store.subscribe(() => {
          data.current = undefined;
        });
      }
      count.current = count.current + 1;
      const handle = store.subscribe(subscriber);
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

  const getSnapshot = useCallback(() => {
    if (data.current === undefined) {
      const favourites = store.get_favourites();
      data.current = favourites;
    }
    return data.current;
  }, [store]);

  const snapshot = useSyncExternalStore(subscribe, getSnapshot, getSnapshot);

  return snapshot;
}

export function useIsFavouritePreference(
  store: WbblWebappPreferencesStore,
  type: WbblWebappNodeType | undefined,
  isOpen: boolean,
): boolean {
  const data = useRef<boolean | undefined>(undefined);
  const count = useRef<number>(0);
  const cacheHandle = useRef<number>(0);

  const subscribe = useCallback(
    (subscriber: () => void) => {
      if (type) {
        if (count.current == 0) {
          cacheHandle.current = store.subscribe(() => {
            data.current = undefined;
          });
        }
        count.current = count.current + 1;
        const handle = store.subscribe(subscriber);
        return () => {
          count.current = count.current - 1;
          if (count.current === 0) {
            store.unsubscribe(cacheHandle.current);
          }
          store.unsubscribe(handle);
        };
      }
      return () => {};
    },
    [store, type],
  );

  useEffect(() => {
    if (type) {
      data.current = store.is_favourite(type);
    }
  }, [type, isOpen]);

  const getSnapshot = useCallback(() => {
    if (data.current === undefined && type) {
      const favourite = store.is_favourite(type);
      data.current = favourite;
    }
    return data.current;
  }, [store, type]);

  const snapshot = useSyncExternalStore(subscribe, getSnapshot, getSnapshot);

  return !!snapshot;
}

export function useKeyBinding(
  store: WbblWebappPreferencesStore,
  shortcut: KeyboardShortcut,
): string | undefined | null {
  const data = useRef<string | undefined | null>(undefined);
  const count = useRef<number>(0);
  const cacheHandle = useRef<number>(0);

  const subscribe = useCallback(
    (subscriber: () => void) => {
      if (count.current == 0) {
        cacheHandle.current = store.subscribe(() => {
          data.current = undefined;
        });
      }
      count.current = count.current + 1;
      const handle = store.subscribe(subscriber);
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

  const getSnapshot = useCallback(() => {
    if (data.current === undefined) {
      data.current = store.get_keybinding(shortcut) ?? null;
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
  const data = useRef<string | undefined | null>(undefined);
  const count = useRef<number>(0);
  const cacheHandle = useRef<number>(0);

  const subscribe = useCallback(
    (subscriber: () => void) => {
      if (count.current == 0) {
        cacheHandle.current = store.subscribe(() => {
          data.current = undefined;
        });
      }
      count.current = count.current + 1;
      const handle = store.subscribe(subscriber);
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
    data.current = store.get_node_keybinding(node_type) ?? null;
  }, [node_type]);

  const getSnapshot = useCallback(() => {
    if (data.current === undefined) {
      data.current = store.get_node_keybinding(node_type) ?? null;
    }
    return data.current;
  }, [store, node_type]);

  const snapshot = useSyncExternalStore(subscribe, getSnapshot, getSnapshot);

  return snapshot;
}

export function useEdgeStyle(store: WbblWebappPreferencesStore): EdgeStyle {
  const data = useRef<EdgeStyle>();
  const count = useRef<number>(0);
  const cacheHandle = useRef<number>(0);

  const subscribe = useCallback(
    (subscriber: () => void) => {
      if (count.current == 0) {
        cacheHandle.current = store.subscribe(() => {
          data.current = undefined;
        });
      }
      count.current = count.current + 1;
      const handle = store.subscribe(subscriber);
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

  const getSnapshot = useCallback(() => {
    if (data.current === undefined) {
      const edgeStyle = store.get_edge_style();
      if (edgeStyle === EdgeStyle.Default) {
        data.current = EdgeStyle.Default;
      } else if (edgeStyle === EdgeStyle.Bezier) {
        data.current = EdgeStyle.Bezier;
      } else {
        data.current = EdgeStyle.Metropolis;
      }
    }
    return data.current;
  }, [store]);

  const snapshot = useSyncExternalStore(subscribe, getSnapshot, getSnapshot);

  return snapshot;
}

export function useIsWobbleEffectEnabledInPreferences(
  store: WbblWebappPreferencesStore,
): boolean {
  const data = useRef<boolean>();
  const count = useRef<number>(0);
  const cacheHandle = useRef<number>(0);

  const subscribe = useCallback(
    (subscriber: () => void) => {
      if (count.current == 0) {
        cacheHandle.current = store.subscribe(() => {
          data.current = undefined;
        });
      }
      count.current = count.current + 1;
      const handle = store.subscribe(subscriber);
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

  const getSnapshot = useCallback(() => {
    if (data.current === undefined) {
      data.current = store.get_allow_wobble() ?? false;
    }
    return data.current;
  }, [store]);

  const snapshot = useSyncExternalStore(subscribe, getSnapshot, getSnapshot);

  return snapshot;
}
