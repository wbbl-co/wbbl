import { NodeProps } from "@xyflow/react";
import WbblNode from "./WbbleNode";
import SourcePort from "../SourcePort";
import { memo } from "react";
import { areNodePropsEqual } from "../../hooks/use-wbbl-graph-store";

function SlabNode(props: NodeProps) {
  return (
    <WbblNode
      outputPorts={<SourcePort id="s#0" key="s#0" />}
      inputPorts={<></>}
      w={200}
      h={200}
      {...props}
    >
      <div></div>
    </WbblNode>
  );
}

export default memo(SlabNode, areNodePropsEqual);
