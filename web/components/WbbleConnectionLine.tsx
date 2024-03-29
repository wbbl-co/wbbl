import { useEffect, useMemo, useRef, useState } from "react";
import { ConnectionLineComponentProps } from "@xyflow/react";
import { WbblRope } from "../../pkg/wbbl";
import { usePortTypeWithNodeId } from "../hooks/use-port-type";
import { getStyleForType } from "../port-type-styling";
import {
  EDGE_STROKE_WIDTH,
  HALF_PORT_SIZE,
  VECTOR_EDGE_STROKE_WIDTH,
} from "../port-constants";

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
        const angle = Math.atan2(
          props.toY - props.fromY,
          props.toX - props.fromX,
        );
        const cosAngle = Math.cos(angle);
        const sinAngle = Math.sin(angle);
        const factorX = -sinAngle;
        const factorY = cosAngle;
        if (
          !!connectionLineClassName &&
          connectionLineClassName.includes("S2")
        ) {
          pathRef.current.style.strokeWidth =
            String(VECTOR_EDGE_STROKE_WIDTH) + "px";
          pathRef.current.setAttribute(
            "d",
            `${rope.get_path(
              new Float32Array([
                -factorX * VECTOR_EDGE_STROKE_WIDTH * 1.5,
                -factorY * 1.5 * VECTOR_EDGE_STROKE_WIDTH,
              ]),
              1,
            )} ${rope.get_path(
              new Float32Array([
                factorX * VECTOR_EDGE_STROKE_WIDTH * 1.5,
                factorY * 1.5 * VECTOR_EDGE_STROKE_WIDTH,
              ]),
              1,
            )}`,
          );
        } else if (
          !!connectionLineClassName &&
          connectionLineClassName.includes("S3")
        ) {
          pathRef.current.style.strokeWidth =
            String(VECTOR_EDGE_STROKE_WIDTH) + "px";
          pathRef.current.setAttribute(
            "d",
            `${rope.get_path(
              new Float32Array([
                -factorX * VECTOR_EDGE_STROKE_WIDTH * 2.5,
                -factorY * VECTOR_EDGE_STROKE_WIDTH * 2.5,
              ]),
              1,
            )} ${rope.get_path(new Float32Array([0, 0]), 1)} ${rope.get_path(
              new Float32Array([
                factorX * VECTOR_EDGE_STROKE_WIDTH * 2.5,
                factorY * VECTOR_EDGE_STROKE_WIDTH * 2.5,
              ]),
              1,
            )}`,
          );
        } else if (
          !!connectionLineClassName &&
          connectionLineClassName.includes("S4")
        ) {
          pathRef.current.style.strokeWidth =
            String(VECTOR_EDGE_STROKE_WIDTH) + "px";
          pathRef.current.setAttribute(
            "d",
            `${rope.get_path(
              new Float32Array([
                -factorX * 4 * VECTOR_EDGE_STROKE_WIDTH,
                -factorY * 4 * VECTOR_EDGE_STROKE_WIDTH,
              ]),
              1,
            )} ${rope.get_path(
              new Float32Array([
                -factorX * 1.5 * VECTOR_EDGE_STROKE_WIDTH,
                -factorY * 1.5 * VECTOR_EDGE_STROKE_WIDTH,
              ]),
              1,
            )} ${rope.get_path(
              new Float32Array([
                factorX * 1.5 * VECTOR_EDGE_STROKE_WIDTH,
                factorY * 1.5 * VECTOR_EDGE_STROKE_WIDTH,
              ]),
              1,
            )} ${rope.get_path(
              new Float32Array([
                factorX * 4 * VECTOR_EDGE_STROKE_WIDTH,
                factorY * 4 * VECTOR_EDGE_STROKE_WIDTH,
              ]),
              1,
            )}`,
          );
        } else {
          pathRef.current.setAttribute(
            "d",
            rope.get_path(new Float32Array([0, 0]), 1),
          );
          pathRef.current.style.strokeWidth = String(EDGE_STROKE_WIDTH) + "px";
        }
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
    connectionLineClassName,
  ]);

  return (
    <>
      <path
        ref={pathRef}
        fill="none"
        className={`react-flow__connection-path rope-path ${connectionLineClassName}`}
        style={{ ...props.connectionLineStyle, transitionProperty: "stroke", transitionDelay: "300ms" }}
      />
      <circle
        className={`start-marker ${connectionLineClassName}`}
        cx={props.fromX}
        cy={props.fromY}
        r={HALF_PORT_SIZE}
      />
      <circle
        className={`end-marker ${connectionLineClassName}`}
        style={{
          filter: "drop-shadow(3px 5px 2px rgb(0 0 0 / 0.4))",
        }}
        cx={props.toX}
        cy={props.toY}
        r={HALF_PORT_SIZE}
      />
    </>
  );
}
