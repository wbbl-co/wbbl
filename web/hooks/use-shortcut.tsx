import {
  HTMLProps,
  PropsWithChildren,
  createContext,
  createElement,
  memo,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
} from "react";
import {
  WbblPreferencesStoreContext,
  useKeyBinding,
} from "./use-preferences-store";
import { KeyboardShortcut } from "../../pkg/wbbl";
import {
  useHotkeys,
  Options as HotkeyOptions,
  useHotkeysContext,
} from "react-hotkeys-hook";
import { AvailableActionsContext } from "./use-actions-menu";

function useShortcut(
  shortcut: KeyboardShortcut,
  callback: () => void,
  dependencies: any[],
  scope: string,
  options?: Omit<HotkeyOptions, "enabled"> & {
    disabled?: boolean;
  },
) {
  const preferencesStore = useContext(WbblPreferencesStoreContext);
  const availableActions = useContext(AvailableActionsContext);
  const keybinding = useKeyBinding(preferencesStore!, shortcut);
  const f = useCallback(callback, dependencies);
  const guardedF = useCallback(() => {
    const handlers = availableActions.actions.get(shortcut) ?? [];
    if (handlers[handlers.length - 1].f === f) {
      f();
    }
  }, [availableActions, shortcut, f]);
  useEffect(() => {
    if (!(options?.disabled ?? false)) {
      let entries: { f: () => void; scope: string }[] = [];
      if (!availableActions.actions.has(shortcut)) {
        availableActions.actions.set(shortcut, entries);
      } else {
        entries = availableActions.actions.get(shortcut)!;
      }
      entries.push({ f, scope });
      entries.sort((a, b) => (a.scope.startsWith(b.scope) ? 1 : -1));
      return () => {
        const entries = availableActions.actions.get(shortcut);
        if (entries) {
          availableActions.actions.set(
            shortcut,
            entries.filter((x) => x.f !== f),
          );
        }
      };
    }
  }, [f, options?.disabled ?? false]);

  const hotkeyOptions = useMemo(
    () => ({
      preventDefault: true,
      ...(options ?? {}),
      enabled: !(options?.disabled ?? false) && !!keybinding,
    }),
    [
      ...Object.entries(options ?? {}).flatMap((kv) => kv),
      keybinding,
      options?.disabled,
    ],
  );
  const hotkeysRef = useHotkeys(
    keybinding ?? "",
    guardedF,
    dependencies,
    hotkeyOptions,
  );

  return hotkeysRef;
}

export const ShortcutScopeContext = createContext<{ scope: string[] }>({
  scope: [],
});

function ShortcutScopeImpl<Type extends keyof JSX.IntrinsicElements = "div">(
  props: PropsWithChildren<
    {
      scope: string;
      as?: Type;
      mode?: "hover" | "focus" | "both";
    } & HTMLProps<JSX.IntrinsicElements[Type]>
  >,
) {
  const { enableScope: enableHotkeysScope, disableScope: disableHotkeysScope } =
    useHotkeysContext();
  const shortcutScope = useContext(ShortcutScopeContext);
  const newScope = useMemo(() => {
    return { scope: [...shortcutScope.scope, props.scope] };
  }, [...shortcutScope.scope, props.scope]);

  const deactivateScopeRef = useRef(() => {});
  const deactivateScope = useCallback(() => {
    disableHotkeysScope(newScope.scope.join("/"));
  }, [disableHotkeysScope, ...newScope.scope]);
  useEffect(() => {
    deactivateScopeRef.current = deactivateScope;
  }, [deactivateScope]);

  const activateScope = useCallback(() => {
    enableHotkeysScope(newScope.scope.join("/"));
  }, [enableHotkeysScope, ...newScope.scope]);

  useEffect(() => {
    return () => {
      deactivateScopeRef.current();
    };
  }, [deactivateScopeRef]);

  return (
    <ShortcutScopeContext.Provider value={newScope}>
      {createElement(
        props.as ?? "div",
        {
          id: `shortcut-scope-${newScope.scope.join("/")}`,
          onBlur:
            props.mode === "both" || props.mode === "focus"
              ? deactivateScope
              : undefined,

          onMouseEnter:
            props.mode === "both" || props.mode === "hover"
              ? activateScope
              : undefined,
          onMouseLeave:
            props.mode === "both" || props.mode === "hover"
              ? deactivateScope
              : undefined,
          onFocus:
            props.mode === "both" || props.mode === "focus"
              ? activateScope
              : undefined,
          tabIndex:
            props.mode === "both" || props.mode === "focus" ? -1 : undefined,
          className: "wrapper-div",
          ...props,
        },
        props.children,
      )}
    </ShortcutScopeContext.Provider>
  );
}

export const ShortcutScope = memo(ShortcutScopeImpl);

export function useScopedShortcut(
  shortcut: KeyboardShortcut,
  callback: () => void,
  dependencies: any[],
  options?: Omit<HotkeyOptions, "enabled" | "scopes"> & { disabled?: boolean },
) {
  const { enabledScopes } = useHotkeysContext();
  const shortcutScope = useContext(ShortcutScopeContext);

  const optionsWithScope = useMemo(() => {
    const thisScope = shortcutScope.scope.join("/");
    const disabled = options?.disabled || !enabledScopes.includes(thisScope);
    return {
      ...options,
      disabled,
    };
  }, [
    ...Object.entries(options ?? {}).flatMap((kv) => kv),
    ...shortcutScope.scope,
    dependencies,
    enabledScopes,
  ]);
  return useShortcut(
    shortcut,
    callback,
    dependencies,
    shortcutScope.scope.join("/"),
    optionsWithScope,
  );
}
