import {
  useNodesState,
  useEdgesState,
  ReactFlow,
  Background,
  Controls,
  MiniMap,
  NodeChange,
  EdgeChange,
  Node,
  Edge,
  ReactFlowProvider,
  Panel,
  Connection,
} from "@xyflow/react";
import React, { useCallback, useContext, useRef } from "react";
import "@xyflow/react/dist/style.css";
import WbbleEdge from "./components/WbbleEdge";
import WbblNode from "./components/WbbleNode";
import WbblConnectionLine from "./components/WbbleConnectionLine";

import {
  WbblGraphStoreContext,
  useWbblGraphData,
} from "./use-wbbl-graph-store";
import { NewWbblWebappNode, WbblWebappGraphStore } from "../pkg/wbbl";

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
  default: WbbleEdge,
};

type WbblWebappNode = Node;
type WbblWebappEdge = Edge;

function Graph() {
  const graphStore = useContext(WbblGraphStoreContext);
  const snapshot = useWbblGraphData(graphStore);
  const addNode = useCallback(
    (evt: React.MouseEvent) => {
      try {
        let node = NewWbblWebappNode.new(evt.clientX, evt.clientY, "default", {
          frog: true,
        });
        graphStore.add_node(node);
      } catch (e) {
        console.error(e);
      }
      evt.preventDefault();
    },
    [graphStore],
  );
  const onNodesChange = useCallback(
    (changes: NodeChange<WbblWebappNode>[]) => {
      for (let change of changes) {
        console.log("node change", change);
        switch (change.type) {
          case "add":
            graphStore.add_node(
              NewWbblWebappNode.new(
                change.item.position.x,
                change.item.position.x,
                change.item.type ?? "default",
                change.item.data,
              ),
            );
            break;
          case "remove":
            graphStore.remove_node(change.id);
            break;
          case "replace":
            graphStore.replace_node(
              NewWbblWebappNode.new(
                change.item.position.x,
                change.item.position.x,
                change.item.type ?? "default",
                change.item.data,
              ),
            );
            break;
          case "dimensions":
            graphStore.set_computed_node_dimension(
              change.id,
              change.dimensions?.width ?? 0,
              change.dimensions?.height ?? 0,
              change.resizing ?? false,
            );
            break;
          case "position":
            graphStore.set_node_position(
              change.id,
              change.position?.x ?? 0,
              change.position?.y ?? 0,
              change.positionAbsolute?.x,
              change.positionAbsolute?.y,
            );
            break;
          case "select":
            graphStore.set_node_selection(change.id, change.selected);
            break;
        }
      }
    },
    [graphStore],
  );

  const onEdgesChange = useCallback(
    (changes: EdgeChange<WbblWebappEdge>[]) => {
      for (let change of changes) {
        console.log("edge change", change);

        switch (change.type) {
          case "add":
            graphStore.add_edge(
              change.item.source,
              change.item.target,
              change.item.sourceHandle!,
              change.item.targetHandle!,
            );
            break;
          case "remove":
            graphStore.remove_edge(change.id);
            break;
          case "replace":
            graphStore.replace_edge(
              change.id,
              change.item.source,
              change.item.target,
              change.item.sourceHandle!,
              change.item.targetHandle!,
              change.item.selected ?? false,
            );
            break;
          case "select":
            graphStore.set_edge_selection(change.id, change.selected);
            break;
        }
      }
    },
    [graphStore],
  );

  const onEdgesUpdate = useCallback(
    (oldEdge: Edge, newConnection: Connection) => {
      graphStore.replace_edge(
        oldEdge.id,
        newConnection.source,
        newConnection.target,
        newConnection.sourceHandle ?? "default",
        newConnection.targetHandle ?? "default",
        false,
      );
    },
    [graphStore],
  );
  const onConnect = useCallback(
    (connection: Connection) => {
      graphStore.add_edge(
        connection.source,
        connection.target,
        connection.sourceHandle ?? "default",
        connection.targetHandle ?? "default",
      );
    },
    [graphStore],
  );

  return (
    <ReactFlow
      nodes={snapshot.nodes as unknown as WbblWebappNode[]}
      onNodesChange={onNodesChange}
      edges={snapshot.edges}
      edgeTypes={edgeTypes}
      nodeTypes={nodeTypes}
      onConnect={onConnect}
      onPaneClick={() => {
        console.log("pane clicked");
      }}
      onEdgesChange={onEdgesChange}
      onEdgeUpdate={onEdgesUpdate}
      onAuxClick={addNode}
      connectionLineComponent={WbblConnectionLine}
      fitView
    >
      <Background />
      <Controls />

      <MiniMap />
    </ReactFlow>
  );
}

function App() {
  const graphStore = useRef(WbblWebappGraphStore.empty());
  // const [nodes, , onNodesChange] = useNodesState(initNodes);
  // const [edges, , onEdgesChange] = useEdgesState(initEdges);

  return (
    <WbblGraphStoreContext.Provider value={graphStore.current}>
      <ReactFlowProvider>
        <Graph />
      </ReactFlowProvider>
    </WbblGraphStoreContext.Provider>
  );
}

export default App;
