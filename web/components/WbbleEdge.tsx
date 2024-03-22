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
  const rope = useMemo(
    () =>
      WbblRope.new(
        new Float32Array([sourceX, sourceY]),
        new Float32Array([targetX, targetY]),
      ),
    [],
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
          x: rectStart.left,
          y: rectStart.top,
        });
        let endPos = flow.screenToFlowPosition({
          x: rectEnd.left,
          y: rectEnd.top,
        });
        if (startMarker.current && endMarker.current) {
          startMarker.current.style.transform = `translate(${rectStart.x + 15 * viewport.zoom}px,${rectStart.y + 7.5 * viewport.zoom}px)`;
          endMarker.current.style.transform = `translate(${rectEnd.x}px,${rectEnd.y + 7.5 * viewport.zoom}px)`;
        }
        rope.update(
          new Float32Array([startPos.x + 7.5, startPos.y + 7.5]),
          new Float32Array([endPos.x + 7.5, endPos.y + 7.5]),
          delta,
        );
        setPath(rope.get_path(new Float32Array([0, 0])));
      }
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
              className={"fill-orange"}
              cx={-7.5 * flow.getZoom()}
              cy="0"
              r={7.5 * flow.getZoom()}
              style={{ filter: "drop-shadow(3px 5px 2px rgb(0 0 0 / 0.4))" }}
            />
            <circle
              ref={endMarker}
              className={"fill-orange shadow-red-50"}
              cx={7.5 * flow.getZoom()}
              cy="0"
              r={7.5 * flow.getZoom()}
              style={{ filter: "drop-shadow(3px 5px 2px rgb(0 0 0 / 0.4))" }}
            />
          </>,
          edgeEnd,
          `edge-marker-${id}`,
        )}
      <BaseEdge
        path={path}
        className="shadow-lg shadow-red-50"
        style={{
          strokeWidth: 4,
          stroke: "#FFD92D",
        }}
      ></BaseEdge>
    </>
  );
}
