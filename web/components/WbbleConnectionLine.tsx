import { useEffect, useMemo, useRef, useState } from "react";
import { ConnectionLineComponentProps } from "@xyflow/react";
import { WbblRope } from "../../pkg/wbbl";
import { usePortTypeWithNodeId } from "../hooks/use-port-type";
import { getStyleForType } from "../port-type-styling";

export default function WbblConnectionLine(
  props: ConnectionLineComponentProps,
) {
  const pathRef = useRef<SVGPathElement | null>(null);
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

  const [rope] = useState(() =>
    WbblRope.new(
      new Float32Array([props.fromX, props.fromY]),
      new Float32Array([props.toX, props.toY]),
    ),
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

      if (pathRef.current) {
        pathRef.current.setAttribute(
          "d",
          rope.get_path(new Float32Array([0, 0]), 1),
        );
      }
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
    pathRef,
  ]);

  return (
    <>
      <path
        ref={pathRef}
        fill="none"
        className={`react-flow__connection-path rope-path ${connectionLineClassName}`}
        style={{ ...props.connectionLineStyle, strokeWidth: 4 }}
      />
      <circle
        className={`start-marker ${connectionLineClassName}`}
        cx={props.fromX}
        cy={props.fromY}
        r={7.5}
      />
      <circle
        className={`end-marker ${connectionLineClassName}`}
        style={{
          filter: "drop-shadow(3px 5px 2px rgb(0 0 0 / 0.4))",
        }}
        cx={props.toX}
        cy={props.toY}
        r={7.5}
      />
    </>
  );
}
