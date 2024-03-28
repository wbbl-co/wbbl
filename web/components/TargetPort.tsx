import {
  Handle,
  Position,
  ReactFlowState,
  useHandleConnections,
  useStore,
} from "@xyflow/react";
import { memo, useMemo } from "react";
import usePortType, { usePortTypeWithNodeId } from "../hooks/use-port-type";
import { WbblWebappGraphStore } from "../../pkg/wbbl";
import { getStyleForType } from "../port-type-styling";

const selector = (s: ReactFlowState) => ({
  nodeInternals: s.nodes,
  edges: s.edges,
  handle: s.connectionStartHandle,
});

type TargetPortProps = { id: `t#${number}`; label?: string; top: number };
function TargetPort(props: TargetPortProps) {
  const { handle } = useStore(selector);
  const portType = usePortType(props.id);

  const handlePortType = usePortTypeWithNodeId(
    handle?.nodeId,
    handle?.handleId as undefined | `s#${number}`,
  );

  const connections = useHandleConnections({
    type: "target",
    id: props.id,
  });

  const isHandleConnectable = useMemo(() => {
    let result =
      connections.length == 0 &&
      !!handlePortType &&
      !!portType &&
      WbblWebappGraphStore.are_port_types_compatible(portType, handlePortType);
    return result;
  }, [connections, portType, handlePortType]);

  return (
    <>
      <Handle
        type="target"
        id={props.id}
        position={Position.Left}
        style={{
          width: 15,
          height: 15,
          borderWidth: 2,
          left: 15,
          top: props.top,
          position: "absolute",
        }}
        isConnectableStart={false}
        className={` ${getStyleForType(portType)} bg-transparent ${isHandleConnectable ? "outline outline-green" : " "}`}
        isConnectable={isHandleConnectable}
      />
      {props.label && (
        <div
          style={{ top: props.top - 10, left: 30 }}
          className="absolute text-left font-mono text-sm italic"
        >
          {props.label}
        </div>
      )}
    </>
  );
}

function propsAreEqual(
  oldProps: TargetPortProps,
  newProps: TargetPortProps,
): boolean {
  return (
    oldProps.id === newProps.id &&
    newProps.label === oldProps.label &&
    oldProps.top === newProps.top
  );
}

export default memo(TargetPort, propsAreEqual);
