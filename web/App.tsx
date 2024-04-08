import { useEffect, useMemo, useState } from "react";
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

function App() {
  let [ready, setReady] = useState<boolean>(false);
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

  return (
    <WbblPreferencesStoreContext.Provider value={preferencesStore}>
      {ready ? (
        <Theme
          appearance={currentTheme == BaseTheme.Dark ? "dark" : "light"}
          accentColor="lime"
          grayColor="gray"
        >
          <div style={{ height: "100vh" }}>
            <ApplicationMenu />
            <Graph />
          </div>
        </Theme>
      ) : (
        <Theme
          appearance={currentTheme == BaseTheme.Dark ? "dark" : "light"}
          accentColor="lime"
          grayColor="gray"
        >
          <LoadingScreen />
        </Theme>
      )}
    </WbblPreferencesStoreContext.Provider>
  );
}

export default App;
