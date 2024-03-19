import {
  Handle,
  Position,
  Node,
  NodeProps,
  useUpdateNodeInternals,
} from "@xyflow/react";
import React, { memo, useContext, useEffect, useRef, useState } from "react";
import { WbblBox } from "../../pkg/wbbl";
import {
  WbblGraphStoreContext,
  useWbblGraphData,
} from "../hooks/use-wbbl-graph-store";

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
      new Float32Array([positionAbsoluteX + 100, positionAbsoluteY + 100]),
      new Float32Array([200, 200]),
    ),
  );
  const graphStore = useContext(WbblGraphStoreContext);
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
    <div style={{ width: 200, height: 200, overflow: "visible" }}>
      <canvas
        style={{ left: -100, top: -100 }}
        className="nodrag absolute"
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
        <Handle
          type="target"
          id={`15272535-e6e7-46c7-8cca-5923e9b179c6`}
          position={Position.Left}
          style={{ background: "white", width: 20, height: 20, left: 20 }}
          isConnectable={true}
        />
        <Handle
          type="source"
          id={`ad9b72b6-38ea-43a1-ae90-ae0f7ce183e3`}
          position={Position.Right}
          style={{
            background: "white",
            width: 20,
            right: 20,
            height: 20,
          }}
          isConnectable={true}
        />
      </div>
    </div>
  );
}
