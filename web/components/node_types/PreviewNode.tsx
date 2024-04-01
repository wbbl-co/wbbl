import { NodeProps } from "@xyflow/react";
import WbblNode from "./WbbleNode";
import { graphWorker } from "../../graph-worker-reference";
import { memo, useLayoutEffect, useState } from "react";
import { DeregisterCanvas, RegisterCanvas } from "../../worker_message_types";
import { areNodePropsEqual } from "../../hooks/use-wbbl-graph-store";

function PreviewNode(props: NodeProps) {
  const [canvasRef, setCanvasRef] = useState<HTMLCanvasElement | null>(null);
  useLayoutEffect(() => {
    if (canvasRef) {
      let offscreenCanvas = canvasRef.transferControlToOffscreen();
      let msg: RegisterCanvas = { nodeId: props.id, offscreenCanvas };
      graphWorker.postMessage({ RegisterCanvas: msg }, [offscreenCanvas]);

      return () => {
        let deregisterMessage: DeregisterCanvas = { nodeId: props.id };
        graphWorker.postMessage({ DeregisterCanvas: deregisterMessage });
      };
    }
  }, [canvasRef, props.id]);

  return (
    <WbblNode
      outputPortLabels={[]}
      inputPortLabels={[null]}
      w={150}
      h={170}
      {...props}
    >
      <canvas
        style={{ backgroundColor: "transparent" }}
        width={128}
        height={128}
        ref={setCanvasRef}
      />
    </WbblNode>
  );
}

export default memo(PreviewNode, areNodePropsEqual);
