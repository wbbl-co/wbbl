import {
  Handle,
  Position,
  ReactFlowState,
  getConnectedEdges,
  useNodeId,
  useStore,
} from "@xyflow/react";
import { useMemo } from "react";

const selector = (s: ReactFlowState) => ({
  nodeInternals: s.nodes,
  edges: s.edges,
});

export default function TargetPort(props: {
  id: string;
  label?: `t-${number}`;
}) {
  const { nodeInternals, edges } = useStore(selector);
  const nodeId = useNodeId();
  const isHandleConnectable = useMemo(() => {
    const node = nodeInternals.find((x) => x.id == nodeId);
    const connectedEdges = getConnectedEdges([node!], edges);
    return (
      connectedEdges.filter(
        (x) => x.target == nodeId && x.targetHandle == props.id,
      ).length == 0
    );
  }, [nodeInternals, edges, nodeId, props.id]);

  return (
    <div className="inline-flex justify-start gap-0 pl-4">
      <Handle
        type="target"
        id={props.id}
        position={Position.Left}
        style={{
          width: 15,
          height: 15,
          borderWidth: 2,
        }}
        className="border-lime relative bg-transparent"
        isConnectable={isHandleConnectable}
      />
      {props.label && (
        <div className="font-mono text-sm italic">{props.label}</div>
      )}
    </div>
  );
}
