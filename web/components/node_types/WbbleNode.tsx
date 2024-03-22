import { NodeProps } from "@xyflow/react";
import { ReactElement, useEffect, useRef, useState } from "react";
import { WbblBox } from "../../../pkg/wbbl";

export default function WbblNode({
  type,
  dragging,
  positionAbsoluteX,
  positionAbsoluteY,
  w,
  h,
  children,
  inputPorts,
  outputPorts,
}: Omit<NodeProps, "width" | "height"> & {
  inputPorts: ReactElement;
  outputPorts: ReactElement;
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
      box.draw(
        context,
        new Float32Array([positionAbsoluteX, positionAbsoluteY]),
      );

      context.globalCompositeOperation = "difference";
      context.closePath();
      context.strokeStyle = "#AB9BF2";
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
  ]);

  return (
    <div
      className="text-sm"
      style={{ width: w, height: h, overflow: "visible" }}
    >
      <canvas
        style={{ left: -(w / 2), top: -(h / 2) }}
        className="nodrag pointer-events-none absolute"
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
        }}
        className="absolute left-0 top-0"
      >
        <div className="pt-1 text-center font-mono text-xl font-bold">
          {type}
        </div>
        {children}
        <div className="absolute left-0 top-0 mt-8 flex w-full flex-row justify-between">
          <div className="top-0 flex flex-col justify-start gap-2">
            {inputPorts}
          </div>
          <div className="flex flex-col gap-2">{outputPorts}</div>
        </div>
      </div>
    </div>
  );
}