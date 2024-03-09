import { Handle, Position, Node, NodeProps } from "@xyflow/react";
import React, { memo, useEffect, useRef, useState } from "react";
import { WbblBox } from "../../pkg/wbbl";

export default function WbblNode({
  id,
  data,
  dragging,
  positionAbsoluteX,
  positionAbsoluteY,
  dragHandle,
}: NodeProps) {
  const [box] = useState(() =>
    WbblBox.new(
      new Float32Array([0, 0]),
      new Float32Array([positionAbsoluteX, positionAbsoluteY]),
    ),
  );
  const canvasRef = useRef<HTMLCanvasElement>(null);

  const lastUpdate = useRef<number>(Date.now());

  useEffect(() => {
    let animationFrame: number;
    function update(time: DOMHighResTimeStamp) {
      let position = canvasRef.current!.getBoundingClientRect();
      const delta = Math.max(0.0, (time - lastUpdate.current) / 1000.0);
      box.update(
        new Float32Array([position.left, position.top]),
        new Float32Array([200, 200]),
        delta,
        dragging ? new Float32Array([position.left, position.top]) : undefined,
      );

      let context = canvasRef.current!.getContext("2d")!;
      context.clearRect(
        0,
        0,
        canvasRef.current!.width,
        canvasRef.current!.height,
      );

      box.draw(context, new Float32Array([position.left, position.top]));
      context.fillStyle = "red";
      context.fill("nonzero");
      context.closePath();
      lastUpdate.current = time;
      animationFrame = requestAnimationFrame(update);
    }
    update(lastUpdate.current);
    return () => cancelAnimationFrame(animationFrame);
  }, [box, lastUpdate, dragging]);

  return (
    <>
      <Handle
        type="target"
        position={Position.Left}
        style={{ background: "#555" }}
        onConnect={(params) => console.log("handle onConnect", params)}
        isConnectable={true}
      />
      <div>Custom Color Picker Node</div>
      <canvas width={250} height={250} ref={canvasRef} />

      <input className="nodrag" type="color" />
      <Handle
        type="source"
        position={Position.Right}
        id="a"
        style={{ top: 10, background: "#555" }}
        isConnectable={true}
      />
      <Handle
        type="source"
        position={Position.Right}
        id="b"
        style={{ bottom: 10, top: "auto", background: "#555" }}
        isConnectable={true}
      />
    </>
  );
}
