import { NodeProps } from "@xyflow/react";
import WbblNode from "./WbbleNode";
import TargetPort from "../TargetPort";
import { graphWorker } from "../../graph-worker-reference";
import { memo, useLayoutEffect, useState } from "react";
import { DeregisterCanvas, RegisterCanvas } from "../../worker_message_types";
import { areNodePropsEqual } from "../../hooks/use-wbbl-graph-store";
// import { WbblGraphStoreContext } from "../../hooks/use-wbbl-graph-store";

function OutputNode(props: NodeProps) {
  const [canvasRef, setCanvasRef] = useState<HTMLCanvasElement | null>(null);
  // const graphStore = useContext(WbblGraphStoreContext);
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
      outputPorts={<></>}
      inputPorts={
        <>
          <TargetPort id="t#0" key="t#0" />
        </>
      }
      w={300}
      h={300}
      {...props}
    >
      <canvas
        style={{ backgroundColor: "transparent" }}
        width={300}
        height={250}
        ref={setCanvasRef}
      />
    </WbblNode>
  );
}

export default memo(OutputNode, areNodePropsEqual);
