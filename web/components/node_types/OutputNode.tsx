import { NodeProps } from "@xyflow/react";
import WbblNode from "./WbbleNode";
import TargetPort from "../TargetPort";

export default function OutputNode(props: NodeProps) {
  return (
    <WbblNode
      outputPorts={<></>}
      inputPorts={
        <>
          <TargetPort id="t-0" key="t-0" />
        </>
      }
      w={200}
      h={200}
      {...props}
    >
      <div></div>
    </WbblNode>
  );
}
