import { NodeProps } from "@xyflow/react";
import WbblNode from "./WbbleNode";
import TargetPort from "../TargetPort";
import SourcePort from "../SourcePort";

export default function BinaryOperatorNode(props: NodeProps) {
  return (
    <WbblNode
      outputPorts={
        <>
          <SourcePort id="s-0" key="s-0" />
        </>
      }
      inputPorts={
        <>
          <TargetPort id="t-0" key="t-0" label="x" />
          <TargetPort id="t-1" key="t-1" label="y" />
        </>
      }
      w={150}
      h={100}
      {...props}
    >
      <div></div>
    </WbblNode>
  );
}
