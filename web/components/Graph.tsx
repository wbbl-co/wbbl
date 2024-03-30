import {
  NodeChange,
  Node,
  EdgeChange,
  Edge,
  Connection,
  ReactFlow,
  SelectionMode,
  Background,
  BackgroundVariant,
  Controls,
  MiniMap,
  ReactFlowProvider,
  useReactFlow,
} from "@xyflow/react";
import React, {
  useContext,
  useCallback,
  useState,
  useMemo,
  useEffect,
} from "react";
import {
  NewWbblWebappNode,
  WbblWebappGraphStore,
  WbblWebappNodeType,
  from_type_name,
} from "../../pkg/wbbl";
import {
  WbblGraphStoreContext,
  WbblSnapshotContext,
  useWbblGraphData,
} from "../hooks/use-wbbl-graph-store";
import { WbblEdgeEndContext } from "../hooks/use-edge-end-portal";
import WbblConnectionLine from "./WbbleConnectionLine";
import WbbleEdge from "./WbbleEdge";
import NodeMenu, { NODE_MENU_DIMENSIONS } from "./NodeMenu";
import { nodeTypes } from "./node_types";
import { graphWorker } from "../graph-worker-reference";

const edgeTypes = {
  default: WbbleEdge,
};

function Graph() {
  const graphStore = useContext(WbblGraphStoreContext);
  const snapshot = useWbblGraphData(graphStore);
  const [nodeMenuPosition, setNodeMenuPosition] = useState<null | {
    x: number;
    y: number;
    top: number | undefined;
    left: number | undefined;
    right: number | undefined;
    bottom: number | undefined;
  }>(null);
  const [nodeMenuOpen, setNodeMenuOpen] = useState<boolean>(false);
  const flow = useReactFlow();
  const [isConnecting, setIsConnecting] = useState(false);
  const onConnectStart = useCallback(() => {
    setIsConnecting(true);
  }, [setIsConnecting]);
  const onConnectEnd = useCallback(() => {
    // Add a delay, so that the node menu isn't immediately opened
    setTimeout(() => {
      setIsConnecting(false);
    }, 4);
  }, [setIsConnecting]);
  const onPaneClick = useCallback(
    (evt: React.MouseEvent<Element, MouseEvent>) => {
      let target = evt.target as HTMLElement;
      let rect = target.getBoundingClientRect();
      if (nodeMenuOpen === false && !isConnecting) {
        let pos = flow.screenToFlowPosition(
          { x: evt.clientX, y: evt.clientY },
          { snapToGrid: false },
        );
        setNodeMenuOpen(true);
        setNodeMenuPosition({
          x: pos.x,
          y: pos.y,
          top:
            evt.clientY < rect.height - NODE_MENU_DIMENSIONS.height
              ? evt.clientY
              : rect.height - NODE_MENU_DIMENSIONS.height,
          left:
            evt.clientX < rect.width - NODE_MENU_DIMENSIONS.width
              ? evt.clientX
              : rect.width - NODE_MENU_DIMENSIONS.width,
          right: undefined,
          bottom: undefined
        });
      } else {
        setNodeMenuOpen(false);
      }
    },
    [nodeMenuOpen, setNodeMenuPosition, setNodeMenuOpen, flow, isConnecting],
  );
  const onNodesChange = useCallback(
    (changes: NodeChange<Node>[]) => {
      for (let change of changes) {
        switch (change.type) {
          case "add":
            graphStore.add_node(
              NewWbblWebappNode.new(
                change.item.position.x,
                change.item.position.x,
                from_type_name(change.item.type ?? "slab")!,
              ),
            );
            break;
          case "remove":
            graphStore.remove_node(change.id);
            break;
          case "replace":
            graphStore.replace_node(
              NewWbblWebappNode.new_with_data(
                change.item.position.x,
                change.item.position.x,
                from_type_name(change.item.type ?? "slab")!,
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
    (changes: EdgeChange<Edge>[]) => {
      for (let change of changes) {
        switch (change.type) {
          case "add":
            graphStore.add_edge(
              change.item.source,
              change.item.target,
              BigInt(change.item.sourceHandle?.replace("s#", "") ?? "0"),
              BigInt(change.item.targetHandle?.replace("t#", "") ?? "0"),
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
              BigInt(change.item.sourceHandle?.replace("s#", "") ?? "0"),
              BigInt(change.item.targetHandle?.replace("t#", "") ?? "0"),
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
        BigInt(newConnection.sourceHandle?.replace("s#", "") ?? "0"),
        BigInt(newConnection.targetHandle?.replace("t#", "") ?? "0"),
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
        BigInt(connection.sourceHandle?.replace("s#", "") ?? "0"),
        BigInt(connection.targetHandle?.replace("t#", "") ?? "0"),
      );
    },
    [graphStore],
  );

  const addNode = useCallback(
    (type: WbblWebappNodeType, x: number, y: number) => {
      graphStore.add_node(NewWbblWebappNode.new(x, y, type));
    },
    [graphStore],
  );

  const removeEdge = useCallback(
    (_: any, edge: Edge) => {
      graphStore.remove_edge(edge.id);
    },
    [graphStore],
  );

  const [edgeRendererRef, setEdgeRenderRef] = useState<SVGSVGElement | null>(
    null,
  );
  let [boundingRect, setBoundingRect] = useState<DOMRect | null>();
  useEffect(() => {
    setBoundingRect(edgeRendererRef?.parentElement?.getBoundingClientRect());
    const listener = () => {
      setBoundingRect(edgeRendererRef?.parentElement?.getBoundingClientRect());
    };
    edgeRendererRef?.parentElement?.addEventListener("resize", listener);
    return () => {
      edgeRendererRef?.parentElement?.removeEventListener("resize", listener);
    };
  }, [edgeRendererRef, setBoundingRect]);

  let width = boundingRect?.width ?? 1080;
  let height = boundingRect?.height ?? 1920;

  return (
    <WbblSnapshotContext.Provider value={snapshot}>
      <WbblEdgeEndContext.Provider value={edgeRendererRef}>
        <ReactFlow
          nodes={snapshot.nodes}
          onNodesChange={onNodesChange}
          edges={snapshot.edges}
          edgeTypes={edgeTypes}
          nodeTypes={nodeTypes}
          onConnect={onConnect}
          maxZoom={1.4}
          minZoom={0.25}
          snapToGrid={false}
          onPaneClick={onPaneClick}
          onEdgeDoubleClick={removeEdge}
          onConnectStart={onConnectStart}
          onConnectEnd={onConnectEnd}
          onEdgesChange={onEdgesChange}
          onEdgeUpdate={onEdgesUpdate}
          connectionLineComponent={WbblConnectionLine}
          selectionMode={SelectionMode.Partial}
          proOptions={{ hideAttribution: true }}
          fitView
        >
          <Background variant={BackgroundVariant.Dots} />
          <Controls />
          <svg
            id="edge-end-renderer"
            viewBox={`0 0 ${width} ${height}`}
            style={{ width: width, height: height, zIndex: 4, pointerEvents: "none", position: "absolute", left: 0, top: 0 }}
            ref={setEdgeRenderRef}
          ></svg>
          <MiniMap
            pannable
            zoomable
          />
          <NodeMenu
            open={nodeMenuOpen}
            onClose={setNodeMenuOpen}
            position={nodeMenuPosition}
            addNode={addNode}
          />
        </ReactFlow>
      </WbblEdgeEndContext.Provider>
    </WbblSnapshotContext.Provider>
  );
}

export default function GraphRoot() {
  const graphStore = useMemo(() => WbblWebappGraphStore.empty(graphWorker), []);

  return (
    <WbblGraphStoreContext.Provider value={graphStore}>
      <ReactFlowProvider>
        <Graph />
      </ReactFlowProvider>
    </WbblGraphStoreContext.Provider>
  );
}
