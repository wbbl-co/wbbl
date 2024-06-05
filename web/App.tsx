import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  MouseEvent as ReactMouseEvent,
} from "react";
import Graph from "./components/Graph";
import { graphWorker } from "./graph-worker-reference";
import LoadingScreen from "./components/LoadingScreen";
import ApplicationMenu from "./components/ApplicationMenu";
import { Theme } from "@radix-ui/themes";
import { BaseTheme, WbblWebappPreferencesStore } from "../pkg/wbbl";
import {
  WbblPreferencesStoreContext,
  useThemePreferences,
} from "./hooks/use-preferences-store";
import { HotkeysProvider } from "react-hotkeys-hook";
import {
  AvailableActions,
  AvailableActionsContext,
} from "./hooks/use-actions-menu";
import { ShortcutScope } from "./hooks/use-shortcut";
import SearchMenu from "./components/SearchMenu";
import useBlobity from "blobity/lib/react/useBlobity";

function App() {
  useBlobity({
    licenseKey: "DA207C2D-99B04E50-BB399991-ED382D1C",
    magnetic: false,
    zIndex: 1,
    opacity: 0.4,
    kineticMorphing: true,
    focusableElements:
      "[data-blobity], a:not([data-no-blobity]), button:not([data-no-blobity])",
    dotColor: "#FFF",
    color: "#AAF05F",
    mode: "bouncy",
    invert: false,
  });
  const [ready, setReady] = useState<boolean>(false);
  useEffect(() => {
    let timeout_handle: any = 0;
    const listener = (msg: MessageEvent) => {
      if (msg.data === "Ready") {
        clearInterval(timeout_handle);
        graphWorker.removeEventListener("message", listener);
        setReady(true);
      }
    };
    graphWorker.addEventListener("message", listener);
    graphWorker.postMessage("Poll");
    timeout_handle = setInterval(() => {
      graphWorker.postMessage("Poll");
    }, 200);
    return () => {
      graphWorker.removeEventListener("message", listener);
      clearInterval(timeout_handle);
    };
  }, [setReady]);
  const preferencesStore = useMemo(
    () => WbblWebappPreferencesStore.empty(),
    [],
  );

  const { currentTheme } = useThemePreferences(preferencesStore);
  const availableActionsContext: AvailableActions = useMemo(
    () => ({ actions: new Map() }),
    [],
  );
  const mousePosition = useRef({
    x: window.innerWidth / 2,
    y: window.innerHeight / 2,
  });
  const setMousePositionCallback = useCallback(
    (evt: ReactMouseEvent<HTMLDivElement>) => {
      mousePosition.current = { x: evt.clientX, y: evt.clientY };
    },
    [mousePosition],
  );
  const [actionMenuSettings, setActionMenuSettings] = useState({
    open: false,
    useMousePosition: false,
  });

  return (
    <AvailableActionsContext.Provider value={availableActionsContext}>
      <ShortcutScope scope="root">
        <HotkeysProvider initiallyActiveScopes={["root"]}>
          <WbblPreferencesStoreContext.Provider value={preferencesStore}>
            <Theme
              appearance={currentTheme == BaseTheme.Dark ? "dark" : "light"}
              accentColor="lime"
              grayColor="gray"
              onMouseMove={setMousePositionCallback}
            >
              {ready ? (
                <div style={{ height: "100dvh", width: "100dvw" }}>
                  <ApplicationMenu
                    path={[]}
                  />
                  <SearchMenu
                    useMousePosition={actionMenuSettings.useMousePosition}
                    mousePosition={mousePosition}
                    open={actionMenuSettings.open}
                    setActionMenuSettings={setActionMenuSettings}
                  />
                  <Graph />
                </div>
              ) : (
                <LoadingScreen />
              )}
            </Theme>
          </WbblPreferencesStoreContext.Provider>
        </HotkeysProvider>
      </ShortcutScope>
    </AvailableActionsContext.Provider>
  );
}

export default App;
