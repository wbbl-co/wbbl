import { useContext, useEffect, useMemo, useRef, useState } from "react";
import { ConnectionLineComponentProps, useReactFlow } from "@xyflow/react";
import { WbblRope } from "../../pkg/wbbl";
import { createPortal } from "react-dom";
import { WbblEdgeEndContext } from "../hooks/use-edge-end-portal";
import { usePortTypeWithNodeId } from "../hooks/use-port-type";
import { getStyleForType } from "../port-type-styling";

export default function WbblConnectionLine(
  props: ConnectionLineComponentProps,
) {
  const flow = useReactFlow();
  const edgeEnd = useContext(WbblEdgeEndContext);

  const sourceType = usePortTypeWithNodeId(
    props.fromNode?.id,
    props.fromHandle?.id as `${"s" | "t"}#${number}`,
  );
  const connectionLineClassName = useMemo(() => {
    if (sourceType) {
      return getStyleForType(sourceType);
    }
    return "";
  }, [sourceType]);

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
    rope.get_path(new Float32Array([0, 0]), 1),
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
      setPath(rope.get_path(new Float32Array([0, 0]), 1));
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
        className={`react-flow__connection-path rope-path ${connectionLineClassName}`}
        style={{ ...props.connectionLineStyle, strokeWidth: 4 }}
      />
      {edgeEnd != null &&
        createPortal(
          <>
            <circle
              className={`start-marker ${connectionLineClassName}`}
              style={{
                transform: `translate(${startMarkerPos.x}px,${startMarkerPos.y}px)`,
                filter: "drop-shadow(3px 5px 2px rgb(0 0 0 / 0.4))",
                width: 15,
                height: 15,
              }}
              cx="0"
              cy="0"
              r={7.5 * flow.getZoom()}
            />
            <circle
              className={`end-marker ${connectionLineClassName}`}
              style={{
                transform: `translate(${endMarkerPos.x}px,${endMarkerPos.y}px)`,
                filter: "drop-shadow(3px 5px 2px rgb(0 0 0 / 0.4))",
                width: 15 * flow.getZoom(),
                height: 15 * flow.getZoom(),
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
