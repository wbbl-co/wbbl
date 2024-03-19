import React from "react";
import Sidebar from "./components/Sidebar";
import Graph from "./components/Graph";

// const canvasRef = useRef<HTMLCanvasElement>(null);
// const canvasRef2 = useRef<HTMLCanvasElement>(null);

// const initialized = useRef<boolean>(false);
// const [worker, setWorker] = useState<Worker>();
// useEffect(() => {
//   if (!initialized.current && canvasRef.current && canvasRef2.current) {
//     initialized.current = true;
//     let worker = new Worker("/web/worker.ts", {
//       type: "module",
//     });
//     setWorker(worker);
//     let offscreenCanvas = canvasRef.current.transferControlToOffscreen();
//     setTimeout(() => {
//       worker.postMessage({ type: "INIT_NODE", offscreenCanvas }, [
//         offscreenCanvas,
//       ]);
//       setTimeout(() => {
//         console.log(canvasRef2.current);
//         let offscreenCanvas2 =
//           canvasRef2.current!.transferControlToOffscreen();
//         worker.postMessage(
//           { type: "INIT_NODE", offscreenCanvas: offscreenCanvas2 },
//           [offscreenCanvas2],
//         );
//       }, 500);
//     }, 40);
//   }
// }, [initialized, canvasRef.current, canvasRef2.current, setWorker]);
// <div>
//   <canvas
//     id="canvas-1"
//     style={{ backgroundColor: "transparent" }}
//     width={300}
//     height={300}
//     ref={canvasRef}
//   />
//   <canvas
//     id="canvas-2"
//     style={{ backgroundColor: "transparent" }}
//     width={300}
//     height={300}
//     ref={canvasRef2}
//   />
// </div>

function App() {
  return (
    <Sidebar>
      <div style={{ height: "100vh" }}>
        <Graph />
      </div>
    </Sidebar>
  );
}

export default App;
