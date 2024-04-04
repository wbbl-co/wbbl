import { NodeProps } from "@xyflow/react";
import WbblNode from "./WbbleNode";
import { memo } from "react";
import { areNodePropsEqual } from "../../hooks/use-wbbl-graph-store";

function SlabNode(props: NodeProps) {
  return (
    <WbblNode
      deleteable
      copyable
      previewable
      outputPortLabels={[null]}
      inputPortLabels={[]}
      w={200}
      h={200}
      {...props}
    />
  );
}

export default memo(SlabNode, areNodePropsEqual);
