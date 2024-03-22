import { NodeProps } from "@xyflow/react";
import WbblNode from "./WbbleNode";
import TargetPort from "../TargetPort";
import SourcePort from "../SourcePort";

export default function SlabNode(props: NodeProps) {
  return (
    <WbblNode
      outputPorts={
        <>
          <SourcePort id="s-0" key="s-0" />
        </>
      }
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
