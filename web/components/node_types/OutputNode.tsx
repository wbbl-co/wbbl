import { NodeProps } from "@xyflow/react";
import WbblNode from "./WbbleNode";
import { graphWorker } from "../../graph-worker-reference";
import { memo, useLayoutEffect, useState } from "react";
import { DeregisterCanvas, RegisterCanvas } from "../../worker_message_types";
import { areNodePropsEqual } from "../../hooks/use-wbbl-graph-store";
// import { WbblGraphStoreContext } from "../../hooks/use-wbbl-graph-store";

function OutputNode(props: NodeProps) {
  const [canvasRef, setCanvasRef] = useState<HTMLCanvasElement | null>(null);
  useLayoutEffect(() => {
    if (canvasRef) {
      const offscreenCanvas = canvasRef.transferControlToOffscreen();
      const msg: RegisterCanvas = { nodeId: props.id, offscreenCanvas };
      graphWorker.postMessage({ RegisterCanvas: msg }, [offscreenCanvas]);

      return () => {
        const deregisterMessage: DeregisterCanvas = { nodeId: props.id };
        graphWorker.postMessage({ DeregisterCanvas: deregisterMessage });
      };
    }
  }, [canvasRef, props.id]);

  return (
    <WbblNode
      deleteable={false}
      copyable={false}
      previewable={false}
      outputPortLabels={[]}
      inputPortLabels={[null]}
      {...props}
    >
      <canvas
        style={{ backgroundColor: "transparent" }}
        width={256}
        height={256}
        ref={setCanvasRef}
      />
    </WbblNode>
  );
}

export default memo(OutputNode, areNodePropsEqual);
