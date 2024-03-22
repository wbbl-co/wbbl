import { useContext, useEffect, useRef, useState } from "react";
import { ConnectionLineComponentProps, useReactFlow } from "@xyflow/react";
import { WbblRope } from "../../pkg/wbbl";
import { createPortal } from "react-dom";
import { WbblEdgeEndContext } from "../hooks/use-edge-end-portal";

export default function WbblConnectionLine(
  props: ConnectionLineComponentProps,
) {
  const flow = useReactFlow();
  const edgeEnd = useContext(WbblEdgeEndContext);

  const startMarkerPos = flow.flowToScreenPosition({
    x: props.fromX,
    y: props.fromY,
  });
  const endMarkerPos = flow.flowToScreenPosition({
    x: props.toX,
    y: props.toY,
  });

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
    <>
      <path
        d={path}
        fill="none"
        className="react-flow__connection-path"
        style={{ ...props.connectionLineStyle, strokeWidth: 4, stroke: "red" }}
        stroke="red"
      />
      {edgeEnd != null &&
        createPortal(
          <>
            <circle
              fill="red"
              style={{
                transform: `translate(${startMarkerPos.x}px,${startMarkerPos.y}px)`,
                filter: "drop-shadow(3px 5px 2px rgb(0 0 0 / 0.4))",
              }}
              cx="0"
              cy="0"
              r={7.5 * flow.getZoom()}
            />
            <circle
              fill="red"
              style={{
                transform: `translate(${endMarkerPos.x}px,${endMarkerPos.y}px)`,
                filter: "drop-shadow(3px 5px 2px rgb(0 0 0 / 0.4))",
              }}
              cx="0"
              cy="0"
              r={7.5 * flow.getZoom()}
            />
          </>,
          edgeEnd,
          `connection-line`,
        )}
    </>
  );
}
