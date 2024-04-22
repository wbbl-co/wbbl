import { memo, useContext, useEffect, useMemo, useRef, useState } from "react";
import {
  EdgeProps,
  Position,
  getBezierPath,
  getSmoothStepPath,
  getStraightPath,
  useReactFlow,
} from "@xyflow/react";
import { EdgeStyle, WbblRope, WbblWebappGraphStore } from "../../pkg/wbbl";
import { createPortal } from "react-dom";
import { WbblEdgeEndContext } from "../hooks/use-edge-end-portal";
import { usePortTypeWithNodeId } from "../hooks/use-port-type";
import { getStyleForType } from "../port-type-styling";
import { HALF_PORT_SIZE, PORT_SIZE } from "../port-constants";
import {
  defaultConnectionPathProvider,
  setConnectionPath,
} from "../utils/set-connection-path";
import useIsWbblEffectEnabled from "../hooks/use-is-wbble-effect-enabled";
import { PortRefStoreContext } from "../hooks/use-port-location";
import {
  WbblPreferencesStoreContext,
  useEdgeStyle,
} from "../hooks/use-preferences-store";
import EdgeContextMenu from "./NodeOrEdgeContextMenu";
import { ShortcutScope } from "../hooks/use-shortcut";

function WbblInteractiveEdge({
  id,
  sourceX,
  sourceY,
  targetX,
  targetY,
  selected,
  edgeStyle,
  edgeClassName,
}: Pick<
  EdgeProps,
  "id" | "sourceX" | "sourceY" | "targetX" | "targetY" | "selected"
> & { edgeStyle: EdgeStyle; edgeClassName: string }) {
  const [pathRef, setPathRef] = useState<SVGPathElement | null>(null);
  useEffect(() => {
    let path = "";
    if (edgeStyle === EdgeStyle.Default) {
      [path] = getStraightPath({
        sourceX,
        sourceY,
        targetX,
        targetY,
      });
    } else if (edgeStyle === EdgeStyle.Bezier) {
      [path] = getBezierPath({
        sourceX,
        sourceY,
        targetX,
        targetY,
        sourcePosition: Position.Right,
        targetPosition: Position.Left,
      });
    } else {
      [path] = getSmoothStepPath({
        sourceX,
        sourceY,
        targetX,
        targetY,
        sourcePosition: Position.Right,
        targetPosition: Position.Left,
      });
    }
    if (pathRef) {
      pathRef.setAttribute("d", path);
    }
  }, [sourceX, sourceY, targetX, targetY, edgeStyle, pathRef]);

  return (
    <ShortcutScope as="g" scope={`edge-${id}`} mode="hover">
      <EdgeContextMenu
        edgeClassname={edgeClassName}
        isEdge
        id={id}
        selected={!!selected}
      >
        <path
          stroke="red"
          strokeOpacity={0}
          ref={setPathRef}
          fill="none"
          strokeWidth={PORT_SIZE}
        />
      </EdgeContextMenu>
    </ShortcutScope>
  );
}

const MemoWbblInteractiveEdge = memo(WbblInteractiveEdge);

function WbblVisualEdge({
  id,
  sourceX,
  sourceY,
  targetX,
  targetY,
  selected,
  edgeStyle,
  edgeClassName,
  source,
  sourceHandleId,
  target,
  targetHandleId,
}: Pick<
  EdgeProps,
  | "id"
  | "sourceX"
  | "sourceY"
  | "targetX"
  | "targetY"
  | "selected"
  | "target"
  | "targetHandleId"
  | "source"
  | "sourceHandleId"
> & { edgeStyle: EdgeStyle; edgeClassName: string }) {
  const flow = useReactFlow();
  const portRefStore = useContext(PortRefStoreContext);

  const rope = useMemo(
    () =>
      WbblRope.new(
        new Float32Array([sourceX, sourceY]),
        new Float32Array([targetX, targetY]),
      ),
    [],
  );

  const isWbblEffectEnabled = useIsWbblEffectEnabled();

  const startMarker = useRef<SVGCircleElement>(null);
  const endMarker = useRef<SVGCircleElement>(null);
  const ropePath = useRef<SVGPathElement>(null);

  const edgeEnd = useContext(WbblEdgeEndContext);

  const lastUpdate = useRef<number>(Date.now());

  useEffect(() => {
    let animationFrame: number;
    function update(time: DOMHighResTimeStamp) {
      const delta = Math.min(
        0.25,
        Math.max(0.0, (time - lastUpdate.current) / 1000.0),
      );
      let startPos = { x: sourceX - HALF_PORT_SIZE, y: sourceY };
      let endPos = { x: targetX + HALF_PORT_SIZE, y: targetY };
      if (isWbblEffectEnabled) {
        const handleStart = portRefStore.get(`${source}#${sourceHandleId}`);
        const rectStart = handleStart?.getBoundingClientRect();
        if (rectStart && rectStart.width > 0) {
          const startPos1 = flow.screenToFlowPosition(
            {
              x: rectStart.left,
              y: rectStart.top,
            },
            { snapToGrid: false },
          );
          const startPos2 = flow.screenToFlowPosition(
            {
              x: rectStart.right,
              y: rectStart.bottom,
            },
            { snapToGrid: false },
          );
          startPos = {
            x: (startPos1.x + startPos2.x) / 2,
            y: (startPos1.y + startPos2.y) / 2,
          };
        }
        const handleEnd = portRefStore.get(`${target}#${targetHandleId}`);
        const rectEnd = handleEnd?.getBoundingClientRect();
        if (rectEnd && rectEnd.width > 0) {
          const endPos1 = flow.screenToFlowPosition(
            {
              x: rectEnd.left,
              y: rectEnd.top,
            },
            { snapToGrid: false },
          );
          const endPos2 = flow.screenToFlowPosition(
            {
              x: rectEnd.right,
              y: rectEnd.bottom,
            },
            { snapToGrid: false },
          );
          endPos = {
            x: (endPos1.x + endPos2.x) / 2,
            y: (endPos1.y + endPos2.y) / 2,
          };
        }
      }

      if (startMarker.current && endMarker.current) {
        startMarker.current.style.transform = `translate(${startPos.x}px,${startPos.y}px)`;
        endMarker.current.style.transform = `translate(${endPos.x}px,${endPos.y}px)`;
      }
      lastUpdate.current = time;
      let factorX = 0.75;
      let factorY = 0.75;
      if (edgeStyle !== EdgeStyle.Metropolis) {
        const angle = Math.atan2(endPos.y - startPos.y, endPos.x - startPos.x);
        const cosAngle = Math.cos(angle);
        const sinAngle = Math.sin(angle);
        factorX = -sinAngle;
        factorY = cosAngle;
      }
      if (edgeStyle === EdgeStyle.Default && isWbblEffectEnabled) {
        rope.update(
          new Float32Array([startPos.x, startPos.y]),
          new Float32Array([endPos.x, endPos.y]),
          delta,
        );
        if (ropePath.current) {
          setConnectionPath(
            ropePath.current,
            edgeClassName,
            (...args) => rope.get_path(...args),
            factorX,
            factorY,
          );
        }
      } else if (ropePath.current) {
        setConnectionPath(
          ropePath.current,
          edgeClassName,
          defaultConnectionPathProvider(startPos, endPos, edgeStyle),
          factorX,
          factorY,
        );
      }

      if (isWbblEffectEnabled) {
        animationFrame = requestAnimationFrame(update);
      }
    }

    lastUpdate.current = Date.now();
    update(lastUpdate.current);
    return () => cancelAnimationFrame(animationFrame);
  }, [
    flow,
    rope,
    ropePath,
    lastUpdate,
    portRefStore,
    startMarker,
    edgeClassName,
    isWbblEffectEnabled,
    edgeStyle,
    sourceX,
    sourceY,
    targetX,
    targetY,
  ]);

  return (
    edgeEnd != null &&
    createPortal(
      <>
        <path
          ref={ropePath}
          style={{ fill: "none" }}
          className={`rope-path ${selected ? "selected" : ""} ${edgeClassName}`}
        />
        <circle
          ref={startMarker}
          key="start-marker"
          className={`start-marker ${selected ? "selected" : ""} ${edgeClassName}`}
          cx={0}
          cy={0}
          r={HALF_PORT_SIZE}
        />
        <circle
          ref={endMarker}
          key="end-marker"
          className={`end-marker ${selected ? "selected" : ""} ${edgeClassName}`}
          cx={0}
          cy={0}
          r={HALF_PORT_SIZE}
          style={{
            filter: "drop-shadow(3px 5px 2px rgb(0 0 0 / 0.4))",
          }}
        />
      </>,
      edgeEnd,
      `edge-marker-${id}`,
    )
  );
}

const MemoWbblVisualEdge = memo(WbblVisualEdge);

function WbbleEdge({
  id,
  sourceHandleId,
  targetHandleId,
  source,
  target,
  sourceX,
  sourceY,
  targetX,
  targetY,
  selected,
}: EdgeProps) {
  const preferencesStore = useContext(WbblPreferencesStoreContext);
  const edgeStyle = useEdgeStyle(preferencesStore);
  const sourceType = usePortTypeWithNodeId(
    source,
    sourceHandleId as `${"s" | "t"}#${number}`,
  );
  const targetType = usePortTypeWithNodeId(
    target,
    targetHandleId as `${"s" | "t"}#${number}`,
  );
  const edgeType = useMemo(
    () =>
      sourceType &&
      targetType &&
      WbblWebappGraphStore.get_edge_type(sourceType, targetType),
    [sourceType, targetType],
  );
  const edgeClassName = useMemo(() => {
    if (targetType && sourceType) {
      return getStyleForType(edgeType);
    }
    return "";
  }, [edgeType]);

  return (
    <>
      <MemoWbblInteractiveEdge
        id={id}
        sourceX={sourceX}
        sourceY={sourceY}
        targetX={targetX}
        targetY={targetY}
        selected={selected}
        edgeStyle={edgeStyle}
        edgeClassName={edgeClassName}
      />
      <MemoWbblVisualEdge
        id={id}
        sourceX={sourceX}
        sourceY={sourceY}
        targetX={targetX}
        targetY={targetY}
        selected={selected}
        edgeStyle={edgeStyle}
        edgeClassName={edgeClassName}
        source={source}
        target={target}
        sourceHandleId={sourceHandleId}
        targetHandleId={targetHandleId}
      />
    </>
  );
}

export default memo(WbbleEdge);
