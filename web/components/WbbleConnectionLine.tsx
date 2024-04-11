import { useContext, useEffect, useMemo, useRef, useState } from "react";
import { ConnectionLineComponentProps } from "@xyflow/react";
import { EdgeStyle, WbblRope } from "../../pkg/wbbl";
import { usePortTypeWithNodeId } from "../hooks/use-port-type";
import { getStyleForType } from "../port-type-styling";
import { HALF_PORT_SIZE } from "../port-constants";
import { WbblEdgeEndContext } from "../hooks/use-edge-end-portal";
import { createPortal } from "react-dom";
import {
  defaultConnectionPathProvider,
  setConnectionPath,
} from "../utils/set-connection-path";
import useIsWbblEffectEnabled from "../hooks/use-is-wbble-effect-enabled";
import {
  WbblPreferencesStoreContext,
  useEdgeStyle,
} from "../hooks/use-preferences-store";

export default function WbblConnectionLine(
  props: ConnectionLineComponentProps,
) {
  const edgeEnd = useContext(WbblEdgeEndContext);

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
  const isWbblEffectEnabled = useIsWbblEffectEnabled();
  const preferencesStore = useContext(WbblPreferencesStoreContext);
  const edgeStyle = useEdgeStyle(preferencesStore);

  const lastUpdate = useRef<number>(Date.now());

  useEffect(() => {
    let animationFrame: number;
    function update(time: DOMHighResTimeStamp) {
      const delta = Math.min(
        0.5,
        Math.max(0.0, (time - lastUpdate.current) / 1000.0),
      );
      if (startMarker.current && endMarker.current) {
        startMarker.current.style.transform = `translate(${props.fromX}px,${props.fromY}px)`;
        endMarker.current.style.transform = `translate(${props.toX}px,${props.toY}px)`;
      }

      let factorX = 0.75;
      let factorY = 0.75;
      if (edgeStyle !== EdgeStyle.Metropolis) {
        const angle = Math.atan2(
          props.toY - props.fromY,
          props.toX - props.fromX,
        );
        const cosAngle = Math.cos(angle);
        const sinAngle = Math.sin(angle);
        factorX = -sinAngle;
        factorY = cosAngle;
      }

      if (edgeStyle === EdgeStyle.Default && isWbblEffectEnabled) {
        rope.update(
          new Float32Array([props.fromX, props.fromY]),
          new Float32Array([props.toX, props.toY]),
          delta,
        );

        if (pathRef.current) {
          setConnectionPath(
            pathRef.current,
            connectionLineClassName,
            (...args) => rope.get_path(...args),
            factorX,
            factorY,
          );
          lastUpdate.current = time;
          animationFrame = requestAnimationFrame(update);
        }
      } else if (pathRef.current) {
        setConnectionPath(
          pathRef.current,
          connectionLineClassName,
          defaultConnectionPathProvider(
            { x: props.fromX, y: props.fromY },
            { x: props.toX, y: props.toY },
            edgeStyle,
          ),
          factorX,
          factorY,
        );
      }
    }
    update(lastUpdate.current);
    return () => cancelAnimationFrame(animationFrame);
  }, [
    isWbblEffectEnabled,
    edgeStyle,
    rope,
    props.fromX,
    props.fromY,
    props.toX,
    props.toY,
    lastUpdate,
    pathRef,
    connectionLineClassName,
    isWbblEffectEnabled,
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
            r={HALF_PORT_SIZE}
          />
          <circle
            className={`connection end-marker ${connectionLineClassName}`}
            style={{
              filter: "drop-shadow(3px 5px 2px rgb(0 0 0 / 0.4))",
            }}
            ref={endMarker}
            cx={0}
            cy={0}
            r={HALF_PORT_SIZE}
          />
        </>,
        edgeEnd!,
        "connection",
      )}
    </>
  );
}
