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

export default function TargetPort(props: { id: string; label?: string }) {
  const { nodeInternals, edges } = useStore(selector);
  const nodeId = useNodeId();
  const isHandleConnectable = useMemo(() => {
    const node = nodeInternals.find((x) => x.id == nodeId);
    const connectedEdges = getConnectedEdges([node!], edges);
    return (
      connectedEdges.filter(
        (x) => x.targetHandle == nodeId && x.targetHandle == props.id,
      ).length == 0
    );
  }, [nodeInternals, edges, nodeId, props.id]);

  return (
    <div className="inline-flex min-w-12 justify-start gap-0 pl-6">
      <Handle
        type="target"
        id={props.id}
        position={Position.Left}
        style={{
          width: 20,
          height: 20,
          borderWidth: 3,
        }}
        className="border-lime relative bg-transparent"
        isConnectableStart={false}
        isConnectable={isHandleConnectable}
      />
      {props.label && (
        <div className="text-md font-mono italic">{props.label}</div>
      )}
    </div>
  );
}
