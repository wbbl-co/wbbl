import {
  useNodesState,
  useEdgesState,
  ReactFlow,
  Background,
  Controls,
  MiniMap,
} from "@xyflow/react";
import React from "react";
import "@xyflow/react/dist/style.css";
import WbbleEdge from "./components/WbbleEdge";
import WbblNode from "./components/WbbleNode";
const initNodes = [
  {
    id: "a",
    type: "wbbl",
    data: { label: "Node A" },
    position: { x: 250, y: 0 },
  },
  {
    id: "b",
    data: { label: "Node B" },
    position: { x: 100, y: 100 },
  },
];

const initEdges = [
  {
    id: "a-b",
    source: "a",
    target: "b",
    type: "wbbl",
  },
];

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

const nodeTypes = {
  wbbl: WbblNode,
};

const edgeTypes = {
  wbbl: WbbleEdge,
};

function App() {
  const [nodes, , onNodesChange] = useNodesState(initNodes);
  const [edges, , onEdgesChange] = useEdgesState(initEdges);

  return (
    <ReactFlow
      nodes={nodes}
      onNodesChange={onNodesChange}
      edges={edges}
      edgeTypes={edgeTypes}
      nodeTypes={nodeTypes}
      onEdgesChange={onEdgesChange}
      fitView
    >
      <Background />
      <Controls />
      <MiniMap />
    </ReactFlow>
  );
}

export default App;
