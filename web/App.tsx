import { useEffect, useState } from "react";
import Graph from "./components/Graph";
import { graphWorker } from "./graph-worker-reference";
import LoadingScreen from "./components/LoadingScreen";
import ApplicationMenu from "./components/ApplicationMenu";
import { ShortcutScope } from "./hooks/use-shortcut";

function App() {
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

  return (
    <ShortcutScope scope="root">
      {ready ? (
        <div style={{ height: "100dvh", width: "100dvw" }}>
          <ApplicationMenu path={[]} />
          <Graph />
        </div>
      ) : (
        <LoadingScreen />
      )}
    </ShortcutScope>
  );
}

export default App;
