import { NodeProps } from "@xyflow/react";
import WbblNode from "./WbbleNode";
import { memo } from "react";
import { areNodePropsEqual } from "../../hooks/use-wbbl-graph-store";

function BuiltInNode(props: NodeProps) {
  return (
    <WbblNode
      deleteable
      copyable
      previewable
      outputPortLabels={[null]}
      inputPortLabels={[]}
      w={150}
      h={100}
      {...props}
    >
      <div />
    </WbblNode>
  );
}

export default memo(BuiltInNode, areNodePropsEqual);
