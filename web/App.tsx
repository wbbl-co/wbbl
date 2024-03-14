import {
  ReactFlow,
  Controls,
  Background,
  MiniMap,
  useNodesState,
  useEdgesState,
} from "@xyflow/react";
import React, { useEffect, useLayoutEffect, useRef, useState } from "react";
import "@xyflow/react/dist/style.css";
import WbbleEdge from "./components/WbbleEdge";
import WbblNode from "./components/WbbleNode";
import { render_preview } from "../pkg/wbbl";
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

const nodeTypes = {
  wbbl: WbblNode,
};

const edgeTypes = {
  wbbl: WbbleEdge,
};

function App() {
  const [nodes, , onNodesChange] = useNodesState(initNodes);
  const [edges, , onEdgesChange] = useEdgesState(initEdges);

  const canvasRef = useRef<HTMLCanvasElement>(null);
  const rendered = useRef<boolean>(false);
  useEffect(() => {
    if (!rendered.current && canvasRef.current) {
      rendered.current = true;
      render_preview(canvasRef.current!);
    }
  }, [rendered, canvasRef.current]);
  return (
    <div>
      <canvas
        style={{ backgroundColor: "transparent" }}
        width={300}
        height={300}
        ref={canvasRef}
      />
    </div>
    // <ReactFlow
    //   nodes={nodes}
    //   onNodesChange={onNodesChange}
    //   edges={edges}
    //   edgeTypes={edgeTypes}
    //   nodeTypes={nodeTypes}
    //   onEdgesChange={onEdgesChange}
    //   fitView
    // >
    //   <Background />
    //   <Controls />
    //   <MiniMap />
    // </ReactFlow>
  );
}

export default App;
