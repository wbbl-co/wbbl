import React, { useEffect, useRef, useState } from "react";
import { ConnectionLineComponentProps } from "@xyflow/react";
import ConnectionLine from "@xyflow/react/dist/umd/components/ConnectionLine";
import { WbblRope } from "../../pkg/wbbl";

export default function WbblConnectionLine(
  props: ConnectionLineComponentProps,
) {
  const [rope] = useState(() =>
    WbblRope.new(
      new Float32Array([props.fromX, props.fromY]),
      new Float32Array([props.toX, props.toY]),
    ),
  );
  const [path, setPath] = useState(() =>
    rope.get_path(new Float32Array([0, 0])),
  );

  const lastUpdate = useRef<number>(Date.now());

  useEffect(() => {
    let animationFrame: number;
    function update(time: DOMHighResTimeStamp) {
      const delta = Math.min(
        0.5,
        Math.max(0.0, (time - lastUpdate.current) / 1000.0),
      );
      rope.update(
        new Float32Array([props.fromX, props.fromY]),
        new Float32Array([props.toX, props.toY]),
        delta,
      );
      setPath(rope.get_path(new Float32Array([0, 0])));
      lastUpdate.current = time;
      animationFrame = requestAnimationFrame(update);
    }
    update(lastUpdate.current);
    return () => cancelAnimationFrame(animationFrame);
  }, [
    rope,
    props.fromX,
    props.fromY,
    props.toX,
    props.toY,
    lastUpdate,
    setPath,
  ]);

  return (
    <path
      d={path}
      fill="none"
      className="react-flow__connection-path"
      style={props.connectionLineStyle}
    />
  );
}
