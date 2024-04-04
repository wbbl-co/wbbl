import { useContext, useEffect, useMemo, useRef } from "react";
import {
  BaseEdge,
  EdgeProps,
  getStraightPath,
  useReactFlow,
  useViewport,
} from "@xyflow/react";
import { WbblRope, WbblWebappGraphStore } from "../../pkg/wbbl";
import { createPortal } from "react-dom";
import { WbblEdgeEndContext } from "../hooks/use-edge-end-portal";
import { usePortTypeWithNodeId } from "../hooks/use-port-type";
import { getStyleForType } from "../port-type-styling";
import { HALF_PORT_SIZE } from "../port-constants";
import {
  defaultConnectionPathProvider,
  setConnectionPath,
} from "../utils/set-connection-path";
import useIsWbblEffectEnabled from "../hooks/use-is-wbble-effect-enabled";

export default function WbbleEdge({
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
  const flow = useReactFlow();
  const sourceType = usePortTypeWithNodeId(
    source,
    sourceHandleId as `${"s" | "t"}#${number}`,
  );
  const targetType = usePortTypeWithNodeId(
    target,
    targetHandleId as `${"s" | "t"}#${number}`,
  );
  const edgeClassName = useMemo(() => {
    if (targetType && sourceType) {
      return getStyleForType(
        WbblWebappGraphStore.get_edge_type(sourceType, targetType),
      );
    }
    return "";
  }, [sourceType, targetType]);

  const handleStart = useMemo(() => {
    return document.querySelector(
      `div[data-handleid="${sourceHandleId}"][data-nodeid="${source}"]`,
    );
  }, [sourceHandleId, source]);
  const handleEnd = useMemo(() => {
    return document.querySelector(
      `div[data-handleid="${targetHandleId}"][data-nodeid="${target}"]`,
    );
  }, [targetHandleId, target]);
  const rope = useMemo(
    () =>
      WbblRope.new(
        new Float32Array([sourceX, sourceY]),
        new Float32Array([targetX, targetY]),
      ),
    [],
  );

  const isWbblEffectEnabled = useIsWbblEffectEnabled();

  const pathElement = useMemo(() => {
    const [path] = getStraightPath({
      sourceX,
      sourceY,
      targetX,
      targetY,
    });
    return <BaseEdge path={path} interactionWidth={25} />;
  }, [sourceX, sourceY, targetX, targetY]);

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
      if (handleStart && handleEnd) {
        let rectStart = handleStart.getBoundingClientRect();
        let rectEnd = handleEnd.getBoundingClientRect();
        let startPos1 = flow.screenToFlowPosition(
          {
            x: rectStart.left,
            y: rectStart.top,
          },
          { snapToGrid: false },
        );
        let startPos2 = flow.screenToFlowPosition(
          {
            x: rectStart.right,
            y: rectStart.bottom,
          },
          { snapToGrid: false },
        );
        let startPos = {
          x: (startPos1.x + startPos2.x) / 2,
          y: (startPos1.y + startPos2.y) / 2,
        };
        let endPos1 = flow.screenToFlowPosition(
          {
            x: rectEnd.left,
            y: rectEnd.top,
          },
          { snapToGrid: false },
        );
        let endPos2 = flow.screenToFlowPosition(
          {
            x: rectEnd.right,
            y: rectEnd.bottom,
          },
          { snapToGrid: false },
        );
        let endPos = {
          x: (endPos1.x + endPos2.x) / 2,
          y: (endPos1.y + endPos2.y) / 2,
        };

        if (startMarker.current && endMarker.current) {
          startMarker.current.style.transform = `translate(${startPos.x}px,${startPos.y}px)`;
          endMarker.current.style.transform = `translate(${endPos.x}px,${endPos.y}px)`;
        }
        lastUpdate.current = time;
        const angle = Math.atan2(endPos.y - startPos.y, endPos.x - startPos.x);
        const cosAngle = Math.cos(angle);
        const sinAngle = Math.sin(angle);
        const factorX = -sinAngle;
        const factorY = cosAngle;
        if (isWbblEffectEnabled) {
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
          animationFrame = requestAnimationFrame(update);
        } else if (ropePath.current) {
          setConnectionPath(
            ropePath.current,
            edgeClassName,
            defaultConnectionPathProvider(startPos, endPos),
            factorX,
            factorY,
          );
        }
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
    handleStart,
    handleEnd,
    startMarker,
    edgeClassName,
    isWbblEffectEnabled,
    sourceX,
    sourceY,
    targetX,
    targetY,
  ]);

  const visibleEdges = useMemo(() => {
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
  }, [edgeEnd, selected, edgeClassName, id, ropePath, startMarker, endMarker]);

  return (
    <>
      {visibleEdges}
      {pathElement}
    </>
  );
}
