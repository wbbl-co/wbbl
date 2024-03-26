import { useEffect, useState } from "react";
import Graph from "./components/Graph";
import { graphWorker } from "./graph-worker-reference";
import LoadingScreen from "./components/LoadingScreen";

function App() {
  let [ready, setReady] = useState<boolean>(false);
  useEffect(() => {
    let timeout_handle: any = 0;
    const listener = (msg: MessageEvent) => {
      if (msg.data === "Ready") {
        clearInterval(timeout_handle);
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

  return ready ? (
    <div style={{ height: "100vh" }}>
      <Graph />
    </div>
  ) : (
    <LoadingScreen />
  );
}

export default App;
