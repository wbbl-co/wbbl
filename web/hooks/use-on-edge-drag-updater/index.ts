import {
  pointToRendererPoint,
  rendererPointToPoint,
  getHostForElement,
  calcAutoPan,
  getEventPosition,
  addEdge,
} from "@xyflow/system";
import {
  ConnectionMode,
  type OnConnect,
  type OnConnectStart,
  type HandleType,
  type Connection,
  type PanBy,
  type NodeBase,
  type Transform,
  type ConnectingHandle,
  type OnConnectEnd,
  type UpdateConnection,
  type IsValidConnection,
  type ConnectionHandle,
} from "@xyflow/system";
import { useCallback, MouseEvent as ReactMouseEvent, useState } from "react";
import {
  getClosestHandle,
  getConnectionStatus,
  getHandleLookup,
  getHandleType,
} from "./utils";
import { Edge, useStoreApi } from "@xyflow/react";
import { WbblWebappGraphStore } from "../../../pkg/wbbl";

type OnPointerDownParams = {
  autoPanOnConnect: boolean;
  connectionMode: ConnectionMode;
  connectionRadius: number;
  domNode: HTMLDivElement | null;
  handleId: string | null;
  nodeId: string;
  isTarget: boolean;
  nodes: NodeBase[];
  lib: string;
  flowId: string | null;
  edgeUpdaterType?: HandleType;
  updateConnection: UpdateConnection;
  panBy: PanBy;
  cancelConnection: () => void;
  onConnectStart?: OnConnectStart;
  onConnect?: OnConnect;
  onConnectEnd?: OnConnectEnd;
  isValidConnection?: IsValidConnection;
  onEdgeUpdateEnd?: (evt: MouseEvent | TouchEvent) => void;
  getTransform: () => Transform;
  getConnectionStartHandle: () => ConnectingHandle | null;
};

type IsValidParams = {
  handle: Pick<ConnectionHandle, "nodeId" | "id" | "type"> | null;
  connectionMode: ConnectionMode;
  fromNodeId: string;
  fromHandleId: string | null;
  fromType: HandleType;
  isValidConnection?: IsValidConnection;
  doc: Document | ShadowRoot;
  lib: string;
  flowId: string | null;
};

type Result = {
  handleDomNode: Element | null;
  isValid: boolean;
  connection: Connection | null;
  endHandle: ConnectingHandle | null;
};

const alwaysValid = () => true;

let connectionStartHandle: ConnectingHandle | null = null;

function onPointerDown(
  event: MouseEvent | TouchEvent,
  {
    connectionMode,
    connectionRadius,
    handleId,
    nodeId,
    edgeUpdaterType,
    isTarget,
    domNode,
    nodes,
    lib,
    autoPanOnConnect,
    flowId,
    panBy,
    cancelConnection,
    onConnectStart,
    onConnect,
    onConnectEnd,
    isValidConnection = alwaysValid,
    onEdgeUpdateEnd,
    updateConnection,
    getTransform,
    getConnectionStartHandle,
  }: OnPointerDownParams,
): {
  onPointerMove: (evt: MouseEvent | TouchEvent) => void;
  onPointerUp: (evt: MouseEvent | TouchEvent) => void;
} {
  // when xyflow is used inside a shadow root we can't use document
  const doc = getHostForElement(event.target as HTMLElement);
  let autoPanId = 0;
  let closestHandle: ConnectionHandle | null;

  const { x, y } = getEventPosition(event);
  const clickedHandle = doc?.elementFromPoint(x, y);
  const handleType = getHandleType(edgeUpdaterType, clickedHandle);
  const containerBounds = domNode?.getBoundingClientRect();

  if (!containerBounds || !handleType) {
    return { onPointerMove: () => {}, onPointerUp: () => {} };
  }

  let connectionPosition = getEventPosition(event, containerBounds);
  let autoPanStarted = false;
  let connection: Connection | null = null;
  let isValid = false;
  let handleDomNode: Element | null = null;

  const handleLookup = getHandleLookup({
    nodes,
    nodeId,
    handleId,
    handleType,
  });

  // when the user is moving the mouse close to the edge of the canvas while connecting we move the canvas
  function autoPan(): void {
    if (!autoPanOnConnect || !containerBounds) {
      return;
    }
    const [x, y] = calcAutoPan(connectionPosition, containerBounds);

    panBy({ x, y });
    autoPanId = requestAnimationFrame(autoPan);
  }

  // Stays the same for all consecutive pointermove events
  connectionStartHandle = {
    nodeId,
    handleId,
    type: handleType,
  };

  updateConnection({
    connectionPosition,
    connectionStatus: null,
    // connectionNodeId etc will be removed in the next major in favor of connectionStartHandle
    connectionStartHandle,
    connectionEndHandle: null,
  });

  onConnectStart?.(event, { nodeId, handleId, handleType });

  function onPointerMove(event: MouseEvent | TouchEvent) {
    if (!getConnectionStartHandle()) {
      onPointerUp(event);
    }

    const transform = getTransform();
    connectionPosition = getEventPosition(event, containerBounds);
    closestHandle = getClosestHandle(
      pointToRendererPoint(connectionPosition, transform, false, [1, 1]),
      connectionRadius,
      handleLookup,
    );

    if (!autoPanStarted) {
      autoPan();
      autoPanStarted = true;
    }

    const result = isValidHandle(event, {
      handle: closestHandle,
      connectionMode,
      fromNodeId: nodeId,
      fromHandleId: handleId,
      fromType: isTarget ? "target" : "source",
      isValidConnection,
      doc,
      lib,
      flowId,
    });

    handleDomNode = result.handleDomNode;
    connection = result.connection;
    isValid = result.isValid;

    updateConnection({
      connectionStartHandle,
      connectionPosition:
        closestHandle && isValid
          ? rendererPointToPoint(
              {
                x: closestHandle.x,
                y: closestHandle.y,
              },
              transform,
            )
          : connectionPosition,
      connectionStatus: getConnectionStatus(!!closestHandle, isValid),
      connectionEndHandle: result.endHandle,
    });
  }

  function onPointerUp(event: MouseEvent | TouchEvent) {
    if ((closestHandle || handleDomNode) && connection && isValid) {
      onConnect?.(connection);
    }

    // it's important to get a fresh reference from the store here
    // in order to get the latest state of onConnectEnd
    onConnectEnd?.(event);

    if (edgeUpdaterType) {
      onEdgeUpdateEnd?.(event);
    }

    cancelConnection();
    cancelAnimationFrame(autoPanId);
    autoPanStarted = false;
    isValid = false;
    connection = null;
    handleDomNode = null;
    connectionStartHandle = null;

    doc.removeEventListener("mousemove", onPointerMove as EventListener);
    doc.removeEventListener("mouseup", onPointerUp as EventListener);

    doc.removeEventListener("touchmove", onPointerMove as EventListener);
    doc.removeEventListener("touchend", onPointerUp as EventListener);
  }

  doc.addEventListener("mousemove", onPointerMove as EventListener);
  doc.addEventListener("mouseup", onPointerUp as EventListener);

  doc.addEventListener("touchmove", onPointerMove as EventListener);
  doc.addEventListener("touchend", onPointerUp as EventListener);

  return { onPointerMove, onPointerUp };
}

// checks if  and returns connection in fom of an object { source: 123, target: 312 }
function isValidHandle(
  event: MouseEvent | TouchEvent,
  {
    handle,
    connectionMode,
    fromNodeId,
    fromHandleId,
    fromType,
    doc,
    lib,
    flowId,
    isValidConnection = alwaysValid,
  }: IsValidParams,
) {
  const isTarget = fromType === "target";
  const handleDomNode = handle
    ? doc.querySelector(
        `.${lib}-flow__handle[data-id="${flowId}-${handle?.nodeId}-${handle?.id}-${handle?.type}"]`,
      )
    : null;

  const { x, y } = getEventPosition(event);
  const handleBelow = doc.elementFromPoint(x, y);
  // we always want to prioritize the handle below the mouse cursor over the closest distance handle,
  // because it could be that the center of another handle is closer to the mouse pointer than the handle below the cursor
  const handleToCheck = handleBelow?.classList.contains(`${lib}-flow__handle`)
    ? handleBelow
    : handleDomNode;

  const result: Result = {
    handleDomNode: handleToCheck,
    isValid: false,
    connection: null,
    endHandle: null,
  };

  if (handleToCheck) {
    const handleType = getHandleType(undefined, handleToCheck);
    const handleNodeId = handleToCheck.getAttribute("data-nodeid");
    const handleId = handleToCheck.getAttribute("data-handleid");
    const connectable = handleToCheck.classList.contains("connectable");
    const connectableEnd = handleToCheck.classList.contains("connectableend");

    if (!handleNodeId) {
      return result;
    }

    const connection: Connection = {
      source: isTarget ? handleNodeId : fromNodeId,
      sourceHandle: isTarget ? handleId : fromHandleId,
      target: isTarget ? fromNodeId : handleNodeId,
      targetHandle: isTarget ? fromHandleId : handleId,
    };

    result.connection = connection;

    const isConnectable = connectable && connectableEnd;
    // in strict mode we don't allow target to target or source to source connections
    const isValid =
      isConnectable &&
      (connectionMode === ConnectionMode.Strict
        ? (isTarget && handleType === "source") ||
          (!isTarget && handleType === "target")
        : handleNodeId !== fromNodeId || handleId !== fromHandleId);

    if (isValid) {
      result.endHandle = {
        nodeId: handleNodeId as string,
        handleId,
        type: handleType as HandleType,
      };

      result.isValid = isValidConnection(connection);
    }
  }

  return result;
}

export function useOnEdgeDragUpdater(
  graphStore: WbblWebappGraphStore,
  onConnectEnd: () => {},
) {
  const storeApi = useStoreApi();
  const [connectingHandlers, setConnectingHandlers] = useState<{
    onPointerMove: (evt: ReactMouseEvent) => void;
    onPointerUp: (evt: ReactMouseEvent) => void;
  }>({ onPointerMove: () => {}, onPointerUp: () => {} });

  return {
    ...connectingHandlers,
    onPointerDown: useCallback(
      (evt: ReactMouseEvent, edge: Edge) => {
        if (
          evt.buttons === 1 &&
          storeApi.getState().connectionStartHandle === null
        ) {
          graphStore.remove_edge(edge.id);
          storeApi.setState({
            connectionPosition: { x: evt.clientX, y: evt.clientY },
            connectionStatus: null,
            connectionStartHandle: {
              nodeId: edge.source,
              handleId: edge.sourceHandle!,
              type: "source",
            },
            connectionEndHandle: null,
          });
          const onConnectExtended = (params: Connection) => {
            const {
              defaultEdgeOptions,
              onConnect: onConnectAction,
              hasDefaultEdges,
            } = storeApi.getState();

            const edgeParams = {
              ...defaultEdgeOptions,
              ...params,
            };
            if (hasDefaultEdges) {
              const { edges, setEdges } = storeApi.getState();
              setEdges(addEdge(edgeParams, edges));
            }

            setConnectingHandlers({
              onPointerUp: () => {},
              onPointerMove: () => {},
            });
            onConnectAction?.(edgeParams);
            onConnectEnd?.();
          };
          const currentStore = storeApi.getState();
          const handlers = onPointerDown(evt.nativeEvent, {
            autoPanOnConnect: currentStore.autoPanOnConnect,
            connectionMode: currentStore.connectionMode,
            connectionRadius: currentStore.connectionRadius,
            domNode: currentStore.domNode,
            edgeUpdaterType: "source",
            nodes: currentStore.nodes,
            lib: currentStore.lib,
            isTarget: false,
            handleId: edge.sourceHandle!,
            nodeId: edge.source,
            flowId: currentStore.rfId,
            panBy: currentStore.panBy,
            cancelConnection: () => {
              currentStore.cancelConnection();
              setConnectingHandlers({
                onPointerMove: () => {},
                onPointerUp: () => {},
              });
            },
            onConnectStart: currentStore.onConnectStart,
            onConnectEnd: (arg) => {
              currentStore.onConnectEnd?.(arg);
              setConnectingHandlers({
                onPointerMove: () => {},
                onPointerUp: () => {},
              });
            },
            updateConnection: currentStore.updateConnection,
            onConnect: onConnectExtended,
            isValidConnection: currentStore.isValidConnection,
            getTransform: () => storeApi.getState().transform,
            getConnectionStartHandle: () =>
              storeApi.getState().connectionStartHandle,
          });
          setConnectingHandlers({
            onPointerMove: (evt) => handlers.onPointerMove(evt.nativeEvent),
            onPointerUp: (evt) => handlers.onPointerMove(evt.nativeEvent),
          });
        }
      },
      [graphStore, setConnectingHandlers, storeApi],
    ),
  };
}
