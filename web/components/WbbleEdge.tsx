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
import {
  EDGE_STROKE_WIDTH,
  HALF_PORT_SIZE,
  VECTOR_EDGE_STROKE_WIDTH,
} from "../port-constants";

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
          startMarker.current.style.transform = `translate(${rectStart.x}px,${rectStart.y}px)`;
          endMarker.current.style.transform = `translate(${rectEnd.x}px,${rectEnd.y}px)`;
        }
        rope.update(
          new Float32Array([startPos.x, startPos.y]),
          new Float32Array([endPos.x, endPos.y]),
          delta,
        );
        if (ropePath.current) {
          const angle = Math.atan2(
            endPos.y - startPos.y,
            endPos.x - startPos.x,
          );
          const cosAngle = Math.cos(angle);
          const sinAngle = Math.sin(angle);
          const factorX = -sinAngle;
          const factorY = cosAngle;

          if (!!edgeClassName && edgeClassName.includes("S2")) {
            ropePath.current.style.strokeWidth =
              viewport.zoom * VECTOR_EDGE_STROKE_WIDTH + "px";
            ropePath.current.setAttribute(
              "d",
              `${rope.get_path(
                new Float32Array([
                  viewport.x -
                  factorX * viewport.zoom * 1.5 * VECTOR_EDGE_STROKE_WIDTH,
                  viewport.y -
                  factorY * viewport.zoom * 1.5 * VECTOR_EDGE_STROKE_WIDTH,
                ]),
                viewport.zoom,
              )} ${rope.get_path(
                new Float32Array([
                  viewport.x +
                  factorX * 2 * viewport.zoom * VECTOR_EDGE_STROKE_WIDTH,
                  viewport.y +
                  factorY * 2 * viewport.zoom * VECTOR_EDGE_STROKE_WIDTH,
                ]),
                viewport.zoom,
              )}`,
            );
          } else if (!!edgeClassName && edgeClassName.includes("S3")) {
            ropePath.current.style.strokeWidth =
              viewport.zoom * VECTOR_EDGE_STROKE_WIDTH + "px";
            ropePath.current.setAttribute(
              "d",
              `${rope.get_path(
                new Float32Array([
                  viewport.x -
                  factorX * viewport.zoom * 2.5 * VECTOR_EDGE_STROKE_WIDTH,
                  viewport.y -
                  factorY * viewport.zoom * 2.5 * VECTOR_EDGE_STROKE_WIDTH,
                ]),
                viewport.zoom,
              )} ${rope.get_path(
                new Float32Array([viewport.x, viewport.y]),
                viewport.zoom,
              )} ${rope.get_path(
                new Float32Array([
                  viewport.x +
                  factorX * viewport.zoom * 2.5 * VECTOR_EDGE_STROKE_WIDTH,
                  viewport.y +
                  factorY * viewport.zoom * 2.5 * VECTOR_EDGE_STROKE_WIDTH,
                ]),
                viewport.zoom,
              )}`,
            );
          } else if (!!edgeClassName && edgeClassName.includes("S4")) {
            ropePath.current.style.strokeWidth =
              viewport.zoom * VECTOR_EDGE_STROKE_WIDTH + "px";
            ropePath.current.setAttribute(
              "d",
              `${rope.get_path(
                new Float32Array([
                  viewport.x -
                  factorX * viewport.zoom * 4 * VECTOR_EDGE_STROKE_WIDTH,
                  viewport.y -
                  factorY * viewport.zoom * 4 * VECTOR_EDGE_STROKE_WIDTH,
                ]),
                viewport.zoom,
              )} ${rope.get_path(
                new Float32Array([
                  viewport.x -
                  factorX * viewport.zoom * 1.5 * VECTOR_EDGE_STROKE_WIDTH,
                  viewport.y -
                  factorY * viewport.zoom * 1.5 * VECTOR_EDGE_STROKE_WIDTH,
                ]),
                viewport.zoom,
              )} ${rope.get_path(
                new Float32Array([
                  viewport.x +
                  factorX * viewport.zoom * 1.5 * VECTOR_EDGE_STROKE_WIDTH,
                  viewport.y +
                  factorY * viewport.zoom * 1.5 * VECTOR_EDGE_STROKE_WIDTH,
                ]),
                viewport.zoom,
              )} ${rope.get_path(
                new Float32Array([
                  viewport.x +
                  factorX * viewport.zoom * 4 * VECTOR_EDGE_STROKE_WIDTH,
                  viewport.y +
                  factorY * viewport.zoom * 4 * VECTOR_EDGE_STROKE_WIDTH,
                ]),
                viewport.zoom,
              )}`,
            );
          } else {
            ropePath.current.style.strokeWidth =
              viewport.zoom * EDGE_STROKE_WIDTH + "px";
            ropePath.current.setAttribute(
              "d",
              rope.get_path(
                new Float32Array([viewport.x, viewport.y]),
                viewport.zoom,
              ),
            );
          }
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
    lastUpdate,
    handleStart,
    handleEnd,
    flow,
    startMarker,
    endMarker,
    viewport.x,
    viewport.y,
    viewport.zoom,
    edgeClassName,
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
              cx={HALF_PORT_SIZE * viewport.zoom}
              cy={HALF_PORT_SIZE * viewport.zoom}
              r={HALF_PORT_SIZE * viewport.zoom}
              style={{
                filter: "drop-shadow(3px 5px 2px rgb(0 0 0 / 0.4))",
              }}
            />
            <path
              ref={ropePath}
              style={{ fill: "none" }}
              className={`rope-path fill-none ${edgeClassName}`}
            />
            <circle
              ref={endMarker}
              key="end-marker"
              className={`end-marker ${edgeClassName}`}
              cx={HALF_PORT_SIZE * viewport.zoom}
              cy={HALF_PORT_SIZE * viewport.zoom}
              r={HALF_PORT_SIZE * viewport.zoom}
              style={{
                filter: "drop-shadow(3px 5px 2px rgb(0 0 0 / 0.4))",
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
