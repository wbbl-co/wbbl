import { NodeProps } from "@xyflow/react";
import WbblNode from "./WbbleNode";
import { memo } from "react";
import { areNodePropsEqual } from "../../hooks/use-wbbl-graph-store";

function BinaryOperatorNode(props: NodeProps) {
  return (
    <WbblNode
      deleteable
      copyable
      previewable
      outputPortLabels={[null]}
      inputPortLabels={["x", "y"]}
      {...props}
    />
  );
}

export default memo(BinaryOperatorNode, areNodePropsEqual);
