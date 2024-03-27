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
}: EdgeProps) {
  const flow = useReactFlow();
  const viewport = useViewport();
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

  const [edgePath] = getStraightPath({
    sourceX,
    sourceY,
    targetX,
    targetY,
  });

  const startMarker = useRef<SVGCircleElement>(null);
  const endMarker = useRef<SVGCircleElement>(null);
  const ropePath = useRef<SVGPathElement>(null);
  const typeLabel = useRef<SVGTextElement>(null);

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
        let startPos = flow.screenToFlowPosition({
          x: rectStart.left,
          y: rectStart.top,
        });
        let endPos = flow.screenToFlowPosition({
          x: rectEnd.left,
          y: rectEnd.top,
        });
        if (startMarker.current && endMarker.current && typeLabel.current) {
          startMarker.current.style.transform = `translate(${rectStart.x + 15 * viewport.zoom}px,${rectStart.y + 7.5 * viewport.zoom}px)`;
          endMarker.current.style.transform = `translate(${rectEnd.x}px,${rectEnd.y + 7.5 * viewport.zoom}px)`;
          typeLabel.current.style.transform = `translate(${(rectEnd.x + rectStart.x) / 2}px,${(rectEnd.y + rectStart.y) / 2}px)`;
        }
        rope.update(
          new Float32Array([startPos.x + 7.5, startPos.y + 7.5]),
          new Float32Array([endPos.x + 7.5, endPos.y + 7.5]),
          delta,
        );
        if (ropePath.current) {
          ropePath.current.setAttribute(
            "d",
            rope.get_path(
              new Float32Array([viewport.x, viewport.y]),
              viewport.zoom,
            ),
          );
        }
      }
      lastUpdate.current = time;
      animationFrame = requestAnimationFrame(update);
    }
    update(lastUpdate.current);
    return () => cancelAnimationFrame(animationFrame);
  }, [
    rope,
    ropePath,
    typeLabel,
    lastUpdate,
    handleStart,
    handleEnd,
    flow,
    startMarker,
    endMarker,
    viewport,
  ]);

  return (
    <>
      {edgeEnd != null &&
        createPortal(
          <>
            <circle
              ref={startMarker}
              key="start-marker"
              className={`start-marker ${edgeClassName}`}
              cx={-7.5 * flow.getZoom()}
              cy="0"
              r={7.5 * flow.getZoom()}
              style={{
                filter: "drop-shadow(3px 5px 2px rgb(0 0 0 / 0.4))",
                width: 15,
                height: 15,
              }}
            />
            <path
              ref={ropePath}
              className={`rope-path fill-none ${edgeClassName}`}
              style={{
                strokeWidth: 4 * viewport.zoom,
              }}
            />
            <text ref={typeLabel} fill="white">
              {edgeClassName}
            </text>
            <circle
              ref={endMarker}
              key="end-marker"
              className={`end-marker ${edgeClassName}`}
              cx={7.5 * flow.getZoom()}
              cy="0"
              r={7.5 * flow.getZoom()}
              style={{
                filter: "drop-shadow(3px 5px 2px rgb(0 0 0 / 0.4))",
                width: 15,
                height: 15,
              }}
            />
          </>,
          edgeEnd,
          `edge-marker-${id}`,
        )}
      <BaseEdge
        path={edgePath}
        style={{ zIndex: 100 }}
        className="stroke-transparent shadow-lg shadow-red-50"
        interactionWidth={50}
      ></BaseEdge>
    </>
  );
}
