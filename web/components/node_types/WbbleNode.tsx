import { NodeProps } from "@xyflow/react";
import { ReactElement, useCallback, useEffect, useRef, useState } from "react";
import { WbblBox } from "../../../pkg/wbbl";
import TargetPort from "../TargetPort";
import SourcePort from "../SourcePort";
import { HALF_PORT_SIZE, PORT_SIZE } from "../../port-constants";
import { Text } from "@radix-ui/themes";
import { nodeMetaData } from ".";

function WbblNode({
  type,
  dragging,
  positionAbsoluteX,
  positionAbsoluteY,
  w,
  h,
  children,
  inputPortLabels,
  outputPortLabels,
}: Omit<NodeProps, "width" | "height"> & {
  inputPortLabels: (null | string)[];
  outputPortLabels: (null | string)[];
  w: number;
  h: number;
  children: ReactElement;
}) {
  const [box] = useState(() =>
    WbblBox.new(
      new Float32Array([positionAbsoluteX + h / 2, positionAbsoluteY + h / 2]),
      new Float32Array([w, h]),
    ),
  );

  const canvasRef = useRef<HTMLCanvasElement>(null);
  const contentsRef = useRef<HTMLDivElement>(null);
  const [handleRefs, setHandleRefs] = useState<[HTMLDivElement]>([]);
  const addHandleRef = useCallback((handleRef: HTMLDivElement) => {
    setHandleRefs((prev) => { prev.push(handleRef); return prev; })
  }, [setHandleRefs]);


  const lastUpdate = useRef<number>(Date.now());

  useEffect(() => {
    let animationFrame: number;
    function update(time: DOMHighResTimeStamp) {
      const delta = Math.min(
        0.25,
        Math.max(0.0, (time - lastUpdate.current) / 1000.0),
      );
      box.update(
        new Float32Array([
          positionAbsoluteX + w / 2,
          positionAbsoluteY + h / 2,
        ]),
        new Float32Array([w, h]),
        delta,
        dragging
          ? new Float32Array([positionAbsoluteX, positionAbsoluteY])
          : undefined,
      );

      let context = canvasRef.current!.getContext("2d")!;
      context.clearRect(
        0,
        0,
        canvasRef.current!.width,
        canvasRef.current!.height,
      );

      context.beginPath();
      let radiusFactor = getComputedStyle(canvasRef.current!).getPropertyValue('--radius-factor');
      box.draw(
        context,
        new Float32Array([positionAbsoluteX, positionAbsoluteY]),
        Number(radiusFactor) * 12
      );

      context.closePath();
      context.strokeStyle = getComputedStyle(canvasRef.current!).getPropertyValue(`--${nodeMetaData[type as keyof typeof nodeMetaData].category}-color`);
      context.lineWidth = 4;
      context.stroke();
      let skew = box.get_skew(
        new Float32Array([
          positionAbsoluteX + w / 2,
          positionAbsoluteY + h / 2,
        ]),
      );

      if (contentsRef.current) {
        contentsRef.current.style.transform = skew;
      }
      console.log(handleRefs.length);
      for (let handleRef of handleRefs) {

        handleRef.style.transform = skew;
      }

      lastUpdate.current = time;
      animationFrame = requestAnimationFrame(update);
    }
    update(lastUpdate.current);
    return () => cancelAnimationFrame(animationFrame);
  }, [
    box,
    contentsRef,
    lastUpdate,
    dragging,
    positionAbsoluteX,
    positionAbsoluteY,
    w,
    h,
    type,
    handleRefs
  ]);

  return (
    <div style={{ width: w, height: h, overflow: "visible" }}>
      <canvas
        style={{ left: -(w / 2), top: -(h / 2), pointerEvents: "none", position: "absolute" }}
        className="nodrag wbbl-node-canvas"
        width={w * 2}
        height={h * 2}
        ref={canvasRef}
      />
      <div
        ref={contentsRef}
        style={{
          background: "rgba(0,0,0,0.01)",
          width: w,
          height: h,
          position: "absolute",
          left: 0,
          top: 0
        }}
      >
        <div className="node-type-header">
          <Text>
            {type}
          </Text>
        </div>
        {children}
      </div>
      {inputPortLabels.map((x: string | null, idx: number) => (
        <TargetPort
          top={idx * (PORT_SIZE + HALF_PORT_SIZE) + 45}
          id={`t#${idx}`}
          label={x ?? undefined}
          ref={addHandleRef}
          key={idx}
        />
      ))}
      {outputPortLabels.map((x: string | null, idx: number) => (
        <SourcePort
          top={idx * (PORT_SIZE + HALF_PORT_SIZE) + 45}
          id={`s#${idx}`}
          label={x ?? undefined}
          key={idx}
          width={w}
          ref={addHandleRef}
        />
      ))}
    </div>
  );
}

export default WbblNode;
