import {
  ReactFlow,
  Controls,
  Background,
  MiniMap,
  useNodesState,
  useEdgesState,
} from "@xyflow/react";
import React from "react";
import "@xyflow/react/dist/style.css";
import WbbleEdge from "./components/WbbleEdge";

const initNodes = [
  {
    id: "a",
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
