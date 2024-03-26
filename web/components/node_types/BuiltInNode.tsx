import { NodeProps } from "@xyflow/react";
import WbblNode from "./WbbleNode";
import SourcePort from "../SourcePort";

export default function BuiltInNode(props: NodeProps) {
  return (
    <WbblNode
      outputPorts={
        <>
          <SourcePort id="s-0" key="s-0" />
        </>
      }
      inputPorts={<></>}
      w={150}
      h={100}
      {...props}
    >
      <div></div>
    </WbblNode>
  );
}
