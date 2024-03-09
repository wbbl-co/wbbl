import React, { useEffect, useLayoutEffect, useRef, useState } from "react";
import {
  BaseEdge,
  EdgeLabelRenderer,
  EdgeProps,
  getBezierPath,
  useReactFlow,
} from "@xyflow/react";
import { WbblRope } from "../../pkg/wbbl";

function WbbleEdge({
  sourceX,
  sourceY,
  targetX,
  targetY,
  markerEnd,
}: EdgeProps) {
  const [rope] = useState(() =>
    WbblRope.new(
      new Float32Array([sourceX, sourceY]),
      new Float32Array([targetX, targetY]),
    ),
  );
  const [path, setPath] = useState(() =>
    rope.get_path(new Float32Array([0, 0])),
  );

  const lastUpdate: React.MutableRefObject<null | number> = useRef(null);

  useEffect(() => {
    let animationFrame: number;
    function update(time: DOMHighResTimeStamp) {
      const delta = (time - (lastUpdate.current ?? time)) / 1000.0;
      rope.update(
        new Float32Array([sourceX, sourceY]),
        new Float32Array([targetX, targetY]),
        delta,
      );
      setPath(rope.get_path(new Float32Array([0, 0])));
      lastUpdate.current = time;
      animationFrame = requestAnimationFrame(update);
    }
    update(lastUpdate.current ?? 0);
    animationFrame = requestAnimationFrame(update);
    return () => cancelAnimationFrame(animationFrame);
  }, [sourceX, sourceY, targetX, targetY, lastUpdate]);

  return (
    <>
      <BaseEdge path={path} markerEnd={markerEnd} style={{ stroke: "red" }} />
    </>
  );
}

export default WbbleEdge;
