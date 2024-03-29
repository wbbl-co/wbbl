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
import { PORT_SIZE } from "../port-constants";

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
        key="handle"
        id={props.id}
        position={Position.Left}
        style={{
          width: PORT_SIZE - 4,
          height: PORT_SIZE - 4,
          borderWidth: 2,
          left: PORT_SIZE,
          top: props.top,
          background: 'transparent',
          position: "absolute",
          transitionDuration: "300ms",
          transitionProperty: "stroke"
        }}
        isConnectableStart={false}
        className={` ${getStyleForType(portType)} ${isHandleConnectable ? "glow" : " "}`}
        isConnectable={isHandleConnectable}
      />
      {props.label && (
        <div
          key="label"
          style={{ top: props.top - 10, left: 2 * PORT_SIZE, position: "absolute", textAlign: "left", fontSize: "0.8rem", fontFamily: 'DM Mono', fontStyle: "italic" }}
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
