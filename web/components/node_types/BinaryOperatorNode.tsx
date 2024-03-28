import { NodeProps } from "@xyflow/react";
import WbblNode from "./WbbleNode";
import { memo } from "react";
import { areNodePropsEqual } from "../../hooks/use-wbbl-graph-store";

function BinaryOperatorNode(props: NodeProps) {
  return (
    <WbblNode
      outputPortLabels={[null]}
      inputPortLabels={["x", "y"]}
      w={150}
      h={100}
      {...props}
    >
      <div></div>
    </WbblNode>
  );
}

export default memo(BinaryOperatorNode, areNodePropsEqual);
