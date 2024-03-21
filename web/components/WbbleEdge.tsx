import { useContext, useEffect, useMemo, useRef, useState } from "react";
import { BaseEdge, EdgeProps, useReactFlow, useViewport } from "@xyflow/react";
import { WbblRope } from "../../pkg/wbbl";
import { createPortal } from "react-dom";
import { WbblEdgeEndContext } from "../hooks/use-edge-end-portal";

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
  const viewport = useViewport();
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
  const [rope] = useState(() =>
    WbblRope.new(
      new Float32Array([sourceX, sourceY]),
      new Float32Array([targetX, targetY]),
    ),
  );
  const [path, setPath] = useState(() =>
    rope.get_path(new Float32Array([0, 0])),
  );

  const startMarker = useRef<SVGCircleElement>(null);
  const endMarker = useRef<SVGCircleElement>(null);

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
          x: rectStart.x,
          y: rectStart.y,
        });
        let endPos = flow.screenToFlowPosition({
          x: rectEnd.x,
          y: rectEnd.y,
        });
        if (startMarker.current && endMarker.current) {
          startMarker.current.style.transform = `translate(${rectStart.x + 20 * viewport.zoom}px,${rectStart.y + 10 * viewport.zoom}px)`;
          endMarker.current.style.transform = `translate(${rectEnd.x}px,${rectEnd.y + 10 * viewport.zoom}px)`;
        }
        rope.update(
          new Float32Array([startPos.x + 10, startPos.y + 10]),
          new Float32Array([endPos.x + 10, endPos.y + 10]),
          delta,
        );
      }
      setPath(rope.get_path(new Float32Array([0, 0])));
      lastUpdate.current = time;
      animationFrame = requestAnimationFrame(update);
    }
    update(lastUpdate.current);
    return () => cancelAnimationFrame(animationFrame);
  }, [
    rope,
    lastUpdate,
    setPath,
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
              fill="#FFD92D"
              cx={-10 * flow.getZoom()}
              cy="0"
              r={10 * flow.getZoom()}
            />
            <circle
              ref={endMarker}
              fill="#FFD92D"
              cx={10 * flow.getZoom()}
              cy="0"
              r={10 * flow.getZoom()}
            />
          </>,
          edgeEnd,
          `edge-marker-${id}`,
        )}
      <BaseEdge
        path={path}
        style={{ stroke: selected ? "#FFD92D" : "blue", strokeWidth: 4 }}
      ></BaseEdge>
    </>
  );
}
