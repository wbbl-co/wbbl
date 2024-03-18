import React, { useEffect, useLayoutEffect, useRef, useState } from "react";
import {
  BaseEdge,
  EdgeLabelRenderer,
  EdgeProps,
  getBezierPath,
  useReactFlow,
} from "@xyflow/react";
import { WbblRope } from "../../pkg/wbbl";

export default function WbbleEdge({
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

  const lastUpdate = useRef<number>(Date.now());

  useEffect(() => {
    let animationFrame: number;
    function update(time: DOMHighResTimeStamp) {
      const delta = Math.max(0.0, (time - lastUpdate.current) / 1000.0);
      rope.update(
        new Float32Array([sourceX, sourceY]),
        new Float32Array([targetX, targetY]),
        delta,
      );
      setPath(rope.get_path(new Float32Array([0, 0])));
      lastUpdate.current = time;
      animationFrame = requestAnimationFrame(update);
    }
    update(lastUpdate.current);
    return () => cancelAnimationFrame(animationFrame);
  }, [rope, sourceX, sourceY, targetX, targetY, lastUpdate, setPath]);

  return (
    <>
      <BaseEdge path={path} markerEnd={markerEnd} style={{ stroke: "red" }} />
    </>
  );
}
