import {
  useReactFlow,
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
  Panel,
} from "@xyflow/react";
import React, {
  useContext,
  useCallback,
  useRef,
  useState,
  useMemo,
} from "react";
import { NewWbblWebappNode, WbblWebappGraphStore } from "../../pkg/wbbl";
import {
  WbblGraphStoreContext,
  useWbblGraphData,
} from "../hooks/use-wbbl-graph-store";
import { WbblEdgeEndContext } from "../hooks/use-edge-end-portal";
import WbblConnectionLine from "./WbbleConnectionLine";
import WbblNode from "./WbbleNode";
import WbbleEdge from "./WbbleEdge";
import Breadcrumb from "./Breadcrumb";
import NodeMenu, { NODE_MENU_DIMENSIONS } from "./NodeMenu";

const nodeTypes = {
  default: WbblNode,
};

const edgeTypes = {
  default: WbbleEdge,
};

type WbblWebappNode = Node;
type WbblWebappEdge = Edge;

function Graph() {
  const graphStore = useContext(WbblGraphStoreContext);
  const snapshot = useWbblGraphData(graphStore);
  const flow = useReactFlow();
  const [nodeMenuPosition, setNodeMenuPosition] = useState<null | {
    top: number | undefined;
    left: number | undefined;
    right: number | undefined;
    bottom: number | undefined;
  }>(null);
  const [nodeMenuOpen, setNodeMenuOpen] = useState<boolean>(false);
  const onPaneClick = useCallback(
    (evt: React.MouseEvent<Element, MouseEvent>) => {
      let target = evt.target as HTMLElement;
      let rect = target.getBoundingClientRect();
      if (nodeMenuOpen === false) {
        setNodeMenuOpen(true);
        setNodeMenuPosition({
          top:
            evt.clientY < rect.height - NODE_MENU_DIMENSIONS.height
              ? evt.clientY
              : undefined,
          left:
            evt.clientX < rect.width - NODE_MENU_DIMENSIONS.width
              ? evt.clientX
              : undefined,
          right:
            evt.clientX >= rect.width - NODE_MENU_DIMENSIONS.width
              ? 10
              : undefined,
          bottom:
            evt.clientY >= rect.height - NODE_MENU_DIMENSIONS.height
              ? 10
              : undefined,
        });
      } else {
        setNodeMenuOpen(false);
      }
    },
    [nodeMenuOpen, setNodeMenuPosition, setNodeMenuOpen],
  );
  const onNodesChange = useCallback(
    (changes: NodeChange<WbblWebappNode>[]) => {
      for (let change of changes) {
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
      colorMode="dark"
      nodeTypes={nodeTypes}
      onConnect={onConnect}
      onPaneClick={onPaneClick}
      onEdgesChange={onEdgesChange}
      onEdgeUpdate={onEdgesUpdate}
      connectionLineComponent={WbblConnectionLine}
      selectionMode={SelectionMode.Partial}
      proOptions={{ hideAttribution: true }}
    >
      <Background variant={BackgroundVariant.Dots} bgColor="black" />
      <Controls className="rounded ring-2 ring-neutral-400" />
      <MiniMap className="rounded ring-2 ring-neutral-400" pannable zoomable />
      <Panel position="top-left">
        <Breadcrumb />
      </Panel>
      <NodeMenu
        open={nodeMenuOpen}
        onClose={setNodeMenuOpen}
        position={nodeMenuPosition}
      />
    </ReactFlow>
  );
}

export default function GraphRoot() {
  const graphStore = useRef(WbblWebappGraphStore.empty());
  const [edgeRendererRef, setEdgeRenderRef] = useState<SVGSVGElement | null>(
    null,
  );
  let boundingRect = useMemo(
    () => edgeRendererRef?.parentElement?.getBoundingClientRect(),
    [edgeRendererRef],
  );
  let width = boundingRect?.width ?? 1080;
  let height = boundingRect?.height ?? 1920;
  return (
    <WbblGraphStoreContext.Provider value={graphStore.current}>
      <ReactFlowProvider>
        <WbblEdgeEndContext.Provider value={edgeRendererRef}>
          <Graph />
        </WbblEdgeEndContext.Provider>
        <svg
          id="edge-end-renderer"
          viewBox={`0 0 ${width} ${height}`}
          style={{
            width,
            height,
            top: 0,
            left: 0,
            position: "absolute",
            pointerEvents: "none",
          }}
          ref={setEdgeRenderRef}
        ></svg>
      </ReactFlowProvider>
    </WbblGraphStoreContext.Provider>
  );
}
