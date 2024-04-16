import { useEffect, useRef, useState, createContext, useContext } from "react";
import useIsWbblEffectEnabled from "./use-is-wbble-effect-enabled";
import { WbblBox } from "../../pkg/wbbl";
import { useReactFlow } from "@xyflow/react";

export const MousePositionContext = createContext<{
  current: [number, number];
}>({ current: [0, 0] });

export type UseCardWbblProps = {
  w: number;
  h: number;
  positionAbsoluteX: number;
  positionAbsoluteY: number;
  dragging: boolean;
  selected: boolean;
};

export function useCardWbbl({
  w,
  h,
  positionAbsoluteX,
  positionAbsoluteY,
  selected,
}: UseCardWbblProps) {
  const isWbblEffectEnabled = useIsWbblEffectEnabled();
  const flow = useReactFlow();
  const mousePosition = useContext(MousePositionContext);
  const [box] = useState(() =>
    WbblBox.new(
      new Float32Array([positionAbsoluteX + w / 2, positionAbsoluteY + h / 2]),
      new Float32Array([w, h]),
    ),
  );
  const contentsRef = useRef<HTMLDivElement>(null);
  const lastUpdate = useRef<number>(Date.now());

  useEffect(() => {
    let animationFrame: number;
    function update(time: DOMHighResTimeStamp) {
      const delta = Math.min(
        0.25,
        Math.max(0.0, (time - lastUpdate.current) / 1000.0),
      );
      if (isWbblEffectEnabled) {
        const pos = flow.screenToFlowPosition(
          { x: mousePosition.current[0], y: mousePosition.current[1] },
          { snapToGrid: false },
        );
        box.update(
          new Float32Array([positionAbsoluteX, positionAbsoluteY]),
          new Float32Array([w, h]),
          delta,
          selected ? new Float32Array([pos.x, pos.y]) : undefined,
        );
        if (contentsRef.current) {
          const skew = box.get_skew(new Float32Array([w, h]));
          contentsRef.current.style.transform = skew;
        }

        lastUpdate.current = time;
        animationFrame = requestAnimationFrame(update);
      } else if (contentsRef.current) {
        contentsRef.current.style.transform = "";
      }
    }
    update(lastUpdate.current);
    return () => cancelAnimationFrame(animationFrame);
  }, [
    box,
    contentsRef,
    lastUpdate,
    selected,
    positionAbsoluteX,
    positionAbsoluteY,
    w,
    h,
    mousePosition,
    flow,
    isWbblEffectEnabled,
  ]);

  return contentsRef;
}
