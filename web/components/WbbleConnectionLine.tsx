import { useContext, useEffect, useMemo, useRef, useState } from "react";
import {
  ConnectionLineComponentProps,
  useReactFlow,
  useViewport,
} from "@xyflow/react";
import { WbblRope } from "../../pkg/wbbl";
import { usePortTypeWithNodeId } from "../hooks/use-port-type";
import { getStyleForType } from "../port-type-styling";
import {
  EDGE_STROKE_WIDTH,
  HALF_PORT_SIZE,
  VECTOR_EDGE_STROKE_WIDTH,
} from "../port-constants";
import { WbblEdgeEndContext } from "../hooks/use-edge-end-portal";
import { createPortal } from "react-dom";
import { setConnectionPath } from "../utils/set-connection-path";

export default function WbblConnectionLine(
  props: ConnectionLineComponentProps,
) {
  const edgeEnd = useContext(WbblEdgeEndContext);
  const viewport = useViewport();
  const flow = useReactFlow();

  const startMarker = useRef<SVGCircleElement>(null);
  const endMarker = useRef<SVGCircleElement>(null);

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
      if (startMarker.current && endMarker.current) {
        let startPos = flow.flowToScreenPosition({
          x: props.fromX,
          y: props.fromY,
        });
        let endPos = flow.flowToScreenPosition({ x: props.toX, y: props.toY });
        startMarker.current.style.transform = `translate(${startPos.x}px,${startPos.y}px)`;
        endMarker.current.style.transform = `translate(${endPos.x}px,${endPos.y}px)`;
      }

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
        setConnectionPath(
          pathRef.current,
          viewport,
          connectionLineClassName,
          rope,
          factorX,
          factorY,
        );
        lastUpdate.current = time;
        animationFrame = requestAnimationFrame(update);
      }
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
    viewport,
    flow,
  ]);

  return (
    <>
      {createPortal(
        <>
          <path
            ref={pathRef}
            fill="none"
            className={`connection rope-path ${connectionLineClassName}`}
            style={{
              fill: "none",
              transitionProperty: "stroke",
              transitionDelay: "300ms",
            }}
          />
          <circle
            className={`connection start-marker ${connectionLineClassName}`}
            cx={0}
            cy={0}
            ref={startMarker}
            r={HALF_PORT_SIZE * viewport.zoom}
          />
          <circle
            className={`connection end-marker ${connectionLineClassName}`}
            style={{
              filter: "drop-shadow(3px 5px 2px rgb(0 0 0 / 0.4))",
            }}
            ref={endMarker}
            cx={0}
            cy={0}
            r={HALF_PORT_SIZE * viewport.zoom}
          />
        </>,
        edgeEnd!,
        "connection",
      )}
    </>
  );
}
