import {
  Node,
  Edge,
  Connection,
  ReactFlow,
  SelectionMode,
  Background,
  BackgroundVariant,
  MiniMap,
  ReactFlowProvider,
  useReactFlow,
  OnSelectionChangeFunc,
  OnBeforeDelete,
  useViewport,
  OnNodesChange,
  OnEdgesChange,
} from "@xyflow/react";
import React, {
  useContext,
  useRef,
  useCallback,
  useState,
  useMemo,
  MouseEvent as ReactMouseEvent,
  useEffect,
} from "react";
import {
  KeyboardShortcut,
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
import { nodeMetaData, nodeTypes } from "./node_types";
import { graphWorker } from "../graph-worker-reference";
import { useOnEdgeDragUpdater } from "../hooks/use-on-edge-drag-updater";
import GraphCanvasContextMenu from "./GraphCanvasContextMenu";
import { useScreenDimensions } from "../hooks/use-screen-dimensions";
import { andThen } from "../hooks/and-then";
import { PortRefStore, PortRefStoreContext } from "../hooks/use-port-location";
import { ShortcutScope, useScopedShortcut } from "../hooks/use-shortcut";
import { AvailableActionsContext } from "../hooks/use-actions-menu";
import {
  WbblPreferencesStoreContext,
  useKeyBinding,
} from "../hooks/use-preferences-store";
import { isHotkeyPressed } from "react-hotkeys-hook";
import { transformKeybindingForReactFlow } from "../utils/transform-keybinding-for-react-flow";
import { useElkJs } from "../hooks/use-elkjs";
import { MousePositionContext } from "../hooks/use-card-wbbl";
import { NodeGroupRenderer } from "./NodeGroupRenderer";

const edgeTypes = {
  default: WbbleEdge,
};

function getMinimapNodeClassnames(node: Node) {
  const category =
    nodeMetaData[node.type as keyof typeof nodeMetaData].category;
  return `minimap-node category-${category}`;
}

function Graph() {
  const preferencesStore = useContext(WbblPreferencesStoreContext);
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
  const viewport = useViewport();

  const [isConnecting, setIsConnecting] = useState(false);
  const [isSelecting, setIsSelecting] = useState(false);
  const setIsSelectingTrue = useCallback(
    () => setIsSelecting(true),
    [setIsSelecting],
  );
  const setIsSelectingFalse = useCallback(() => {
    setTimeout(() => setIsSelecting(false), 4);
  }, [setIsSelecting]);

  const onSelectionChange = useCallback<OnSelectionChangeFunc>(
    (selection) => {
      if (selection.nodes.some((x) => x.selected)) {
        setIsSelectingTrue();
      } else {
        setIsSelectingFalse();
      }
    },
    [setIsSelectingTrue, setIsSelectingFalse],
  );
  const selectionKeybinding = useKeyBinding(
    preferencesStore,
    KeyboardShortcut.Selection,
  );
  useScopedShortcut(
    KeyboardShortcut.SelectAll,
    () => {
      graphStore.select_all();
    },
    [graphStore],
  );
  useScopedShortcut(
    KeyboardShortcut.SelectNone,
    () => {
      graphStore.select_none();
    },
    [graphStore],
  );
  const onConnectStart = useCallback(() => {
    setIsConnecting(true);
  }, [setIsConnecting]);
  const onConnectEnd = useCallback(() => {
    // Add a delay, so that the node menu isn't immediately opened
    setTimeout(() => {
      setIsConnecting(false);
    }, 4);
  }, [setIsConnecting]);

  const mousePos = useRef([0, 0] as [number, number]);
  const onMouseMove = useCallback(
    (evt: ReactMouseEvent<HTMLDivElement>) => {
      mousePos.current = [evt.clientX, evt.clientY];
    },
    [mousePos],
  );

  const onPaneClick = useCallback(
    (evt: React.MouseEvent<Element, MouseEvent>) => {
      const target = evt.target as HTMLElement;
      const rect = target.getBoundingClientRect();
      if (!isConnecting && !isSelecting) {
        let nodeAdded = false;
        const nodeKeybindings = preferencesStore.get_node_keybindings() as Map<
          string,
          string | null | undefined
        >;
        for (const keybinding of nodeKeybindings.entries()) {
          if (!!keybinding[1] && isHotkeyPressed(keybinding[1])) {
            nodeAdded = true;
            const pos = flow.screenToFlowPosition(
              { x: mousePos.current[0], y: mousePos.current[1] },
              { snapToGrid: false },
            );
            graphStore.add_node(
              NewWbblWebappNode.new(
                pos.x,
                pos.y,
                nodeMetaData[keybinding[0] as keyof typeof nodeMetaData].type,
              ),
            );
            break;
          }
        }

        if (nodeMenuOpen === false && !nodeAdded) {
          const pos = flow.screenToFlowPosition(
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
            bottom: undefined,
          });
        }
      } else {
        setNodeMenuOpen(false);
      }
    },
    [
      preferencesStore,
      graphStore,
      nodeMenuOpen,
      setNodeMenuPosition,
      setNodeMenuOpen,
      flow,
      isConnecting,
      isSelecting,
      mousePos,
    ],
  );
  const onNodesChange = useCallback<OnNodesChange>(
    (changes) => {
      for (const change of changes) {
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
              true,
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

  const onEdgesChange = useCallback<OnEdgesChange>(
    (changes) => {
      for (const change of changes) {
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
            graphStore.add_edge(
              change.item.source,
              change.item.target,
              BigInt(change.item.sourceHandle?.replace("s#", "") ?? "0"),
              BigInt(change.item.targetHandle?.replace("t#", "") ?? "0"),
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
  const edgeUpdateSuccessful = useRef(true);
  const onEdgesUpdate = useCallback(
    (_: Edge, newConnection: Connection) => {
      edgeUpdateSuccessful.current = true;
      graphStore.add_edge(
        newConnection.source,
        newConnection.target,
        BigInt(newConnection.sourceHandle?.replace("s#", "") ?? "0"),
        BigInt(newConnection.targetHandle?.replace("t#", "") ?? "0"),
      );
    },
    [graphStore],
  );

  const onEdgeUpdateStart = useCallback(
    (_: unknown, edge: Edge) => {
      edgeUpdateSuccessful.current = false;
      graphStore.remove_edge(edge.id);
    },
    [graphStore],
  );

  const onEdgeUpdateEnd = useCallback(
    (_: unknown, edge: Edge) => {
      if (!edgeUpdateSuccessful.current) {
        graphStore.remove_edge(edge.id);
      }
      edgeUpdateSuccessful.current = true;
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

  const availableActionsContext = useContext(AvailableActionsContext);
  useEffect(() => {
    availableActionsContext.addNode = (
      type: WbblWebappNodeType,
      x: number,
      y: number,
    ) => {
      const pos = flow.screenToFlowPosition({ x, y }, { snapToGrid: false });
      graphStore.add_node(NewWbblWebappNode.new(pos.x, pos.y, type));
    };
    return () => {
      availableActionsContext.addNode = undefined;
    };
  }, [availableActionsContext, flow, graphStore]);

  const removeEdge = useCallback(
    (_: any, edge: Edge) => {
      graphStore.remove_edge(edge.id);
    },
    [graphStore],
  );

  const [edgeRendererRef, setEdgeRenderRef] = useState<SVGSVGElement | null>(
    null,
  );

  const connectingHandlers = useOnEdgeDragUpdater(graphStore, onConnectEnd);

  const { width, height } = useScreenDimensions();

  const onBeforeDelete: OnBeforeDelete = useCallback(
    async ({ nodes, edges }) => {
      const selectedNodes = graphStore.get_locally_selected_nodes();
      const selectedEdges = graphStore.get_locally_selected_edges();
      return {
        nodes: nodes.filter((x) => selectedNodes.includes(x.id)),
        edges: edges.filter((x) => selectedEdges.includes(x.id)),
      };
    },
    [graphStore],
  );

  useScopedShortcut(
    KeyboardShortcut.Paste,
    () => {
      const screenPos = flow.screenToFlowPosition(
        { x: mousePos.current[0], y: mousePos.current[1] },
        { snapToGrid: false },
      );
      WbblWebappGraphStore.get_clipboard_snapshot()
        .then((snapshot) =>
          graphStore.integrate_clipboard_snapshot(
            snapshot,
            new Float32Array([screenPos.x, screenPos.y]),
          ),
        )
        .catch(console.error);
    },
    [flow, graphStore, mousePos.current],
  );
  useScopedShortcut(
    KeyboardShortcut.Copy,
    () => {
      graphStore.copy();
    },
    [graphStore],
    {
      disabled:
        snapshot.edges.every((x) => !x.selected) &&
        snapshot.nodes.every((x) => !x.selected),
    },
  );

  useScopedShortcut(
    KeyboardShortcut.Duplicate,
    () => {
      graphStore.duplicate();
    },
    [graphStore, snapshot.edges, snapshot.nodes],
    {
      disabled:
        snapshot.edges.every((x) => !x.selected) &&
        snapshot.nodes.every((x) => !x.selected),
    },
  );

  const elkJs = useElkJs();
  useScopedShortcut(
    KeyboardShortcut.AutoLayout,
    () => {
      elkJs();
    },
    [elkJs],
    {
      disabled: snapshot.nodes.every((x) => !x.selected),
    },
  );

  useScopedShortcut(
    KeyboardShortcut.Cut,
    () => {
      graphStore
        .copy()
        .then(() => {
          graphStore.remove_selected_nodes_and_edges();
        })
        .catch(console.error);
    },
    [graphStore, snapshot.edges, snapshot.nodes],
    {
      disabled:
        snapshot.edges.every((x) => !x.selected) &&
        snapshot.nodes.every((x) => !x.selected),
    },
  );
  useScopedShortcut(
    KeyboardShortcut.Undo,
    () => {
      if (graphStore.can_undo()) {
        graphStore.undo();
      }
    },
    [graphStore],
    {
      disabled: !graphStore.can_undo(),
    },
  );

  useScopedShortcut(
    KeyboardShortcut.Redo,
    () => {
      if (graphStore.can_redo()) {
        graphStore.redo();
      }
    },
    [graphStore],
    {
      disabled: !graphStore.can_redo(),
    },
  );

  useScopedShortcut(
    KeyboardShortcut.Delete,
    () => {
      graphStore.remove_selected_nodes_and_edges();
    },
    [graphStore],
    {
      disabled:
        snapshot.edges.every((x) => !x.selected) &&
        snapshot.nodes.every((x) => !x.selected),
    },
  );

  useScopedShortcut(
    KeyboardShortcut.GroupNodes,
    () => {
      graphStore.group_selected_nodes();
    },
    [graphStore],
    {
      disabled: snapshot.nodes.every((x) => !x.selected),
    },
  );

  useScopedShortcut(
    KeyboardShortcut.UngroupNodes,
    () => {
      graphStore.ungroup_selected_nodes();
    },
    [graphStore],
    {
      disabled: snapshot.nodes.every((x) => !x.selected),
    },
  );

  const portRefStore = useMemo(() => new PortRefStore(), []);
  const sortedNodes = useMemo(() => {
    return snapshot.nodes.sort((a, b) =>
      a.type === b.type ? 0 : a.type === "frame" ? -1 : 1,
    );
  }, [snapshot.nodes]);

  return (
    <MousePositionContext.Provider value={mousePos}>
      <WbblSnapshotContext.Provider value={snapshot}>
        <WbblEdgeEndContext.Provider value={edgeRendererRef}>
          <PortRefStoreContext.Provider value={portRefStore}>
            <GraphCanvasContextMenu mousePosition={mousePos}>
              <ReactFlow
                width={width}
                height={height}
                nodes={sortedNodes}
                onNodesChange={onNodesChange}
                edges={snapshot.edges}
                edgeTypes={edgeTypes}
                nodeTypes={nodeTypes}
                onConnect={onConnect}
                deleteKeyCode={[]}
                maxZoom={1.4}
                minZoom={0.25}
                snapToGrid={false}
                onEdgeMouseMove={connectingHandlers.onPointerDown}
                onPaneClick={onPaneClick}
                onEdgeDoubleClick={removeEdge}
                onConnectStart={onConnectStart}
                onConnectEnd={onConnectEnd}
                onBeforeDelete={onBeforeDelete}
                multiSelectionKeyCode={useMemo(
                  () =>
                    selectionKeybinding
                      ? [transformKeybindingForReactFlow(selectionKeybinding)]
                      : [],
                  [selectionKeybinding],
                )}
                selectionKeyCode={useMemo(
                  () =>
                    selectionKeybinding
                      ? [transformKeybindingForReactFlow(selectionKeybinding)]
                      : [],
                  [selectionKeybinding],
                )}
                onSelectionStart={connectingHandlers.onSelectStart}
                onSelectionEnd={connectingHandlers.onSelectEnd}
                onSelectionChange={onSelectionChange}
                onEdgesChange={onEdgesChange}
                onEdgeUpdate={onEdgesUpdate}
                onEdgeUpdateEnd={onEdgeUpdateEnd}
                onEdgeUpdateStart={onEdgeUpdateStart}
                connectionLineComponent={WbblConnectionLine}
                onPointerMove={andThen(
                  connectingHandlers.onPointerMove,
                  onMouseMove,
                )}
                onPointerUp={connectingHandlers.onPointerMove}
                selectionMode={SelectionMode.Partial}
                proOptions={useMemo(() => ({ hideAttribution: true }), [])}
                fitView
                onlyRenderVisibleElements
              >
                <Background variant={BackgroundVariant.Dots} />
                <NodeGroupRenderer
                  groups={snapshot.node_groups ?? []}
                  width={width}
                  height={height}
                />
                <svg
                  id="edge-end-renderer"
                  style={{
                    width: width,
                    overflow: "visible",
                    height: height,
                    zIndex: 4,
                    pointerEvents: "none",
                    position: "absolute",
                    left: 0,
                    top: 0,
                    transformOrigin: "0 0",
                    transform: `translate(${viewport.x}px, ${viewport.y}px) scale(${viewport.zoom})`,
                  }}
                  ref={setEdgeRenderRef}
                ></svg>
                <MiniMap
                  pannable
                  zoomable
                  nodeClassName={getMinimapNodeClassnames}
                />
                <NodeMenu
                  open={nodeMenuOpen}
                  onClose={setNodeMenuOpen}
                  position={nodeMenuPosition}
                  addNode={addNode}
                />
              </ReactFlow>
            </GraphCanvasContextMenu>
          </PortRefStoreContext.Provider>
        </WbblEdgeEndContext.Provider>
      </WbblSnapshotContext.Provider>
    </MousePositionContext.Provider>
  );
}

export default function GraphRoot() {
  const graphStore = useMemo(() => WbblWebappGraphStore.empty(graphWorker), []);

  return (
    <ShortcutScope
      style={{ width: "100dvw", height: "100dvh" }}
      scope="graph"
      mode="hover"
    >
      <WbblGraphStoreContext.Provider value={graphStore}>
        <ReactFlowProvider>
          <Graph />
        </ReactFlowProvider>
      </WbblGraphStoreContext.Provider>
    </ShortcutScope>
  );
}
