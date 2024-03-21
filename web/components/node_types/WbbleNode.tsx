import { NodeProps } from "@xyflow/react";
import { useEffect, useRef, useState } from "react";
import { WbblBox } from "../../../pkg/wbbl";
import TargetPort from "../TargetPort";
import SourcePort from "../SourcePort";

export default function WbblNode({
  type,
  dragging,
  positionAbsoluteX,
  positionAbsoluteY,
}: NodeProps) {
  const [box] = useState(() =>
    WbblBox.new(
      new Float32Array([positionAbsoluteX + 100, positionAbsoluteY + 100]),
      new Float32Array([200, 200]),
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
        new Float32Array([positionAbsoluteX + 100, positionAbsoluteY + 100]),
        new Float32Array([200, 200]),
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
        new Float32Array([positionAbsoluteX + 100, positionAbsoluteY + 100]),
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
  ]);

  return (
    <div
      className="text-sm"
      style={{ width: 200, height: 200, overflow: "visible" }}
    >
      <canvas
        style={{ left: -100, top: -100 }}
        className="nodrag pointer-events-none absolute"
        width={400}
        height={400}
        ref={canvasRef}
      />
      <div
        ref={contentsRef}
        style={{
          background: "rgba(0,0,0,0.01)",
          width: 200,
          height: 200,
        }}
        className="absolute left-0 top-0"
      >
        <div className="text-center font-mono text-xl font-bold">{type}</div>
        <div className="left-0 top-0 mt-2 flex w-full flex-row justify-between">
          <div className="flex flex-col justify-start gap-2">
            <TargetPort id={`t-0`} label="x" />
            <TargetPort id={`t-1`} label="y" />
          </div>
          <div className="flex flex-col justify-end gap-2">
            <SourcePort id={`s-0`} label="output" />
            <SourcePort id={`s-1`} />
          </div>
        </div>
      </div>
    </div>
  );
}
