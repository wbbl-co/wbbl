import {
  Handle,
  Position,
  ReactFlowState,
  useHandleConnections,
  useNodeId,
  useStore,
} from "@xyflow/react";
import { memo, useContext, useEffect, useMemo, useState } from "react";
import usePortType, { usePortTypeWithNodeId } from "../hooks/use-port-type";
import { WbblWebappGraphStore } from "../../pkg/wbbl";
import { getStyleForType } from "../port-type-styling";
import { PORT_SIZE } from "../port-constants";
import { Text } from "@radix-ui/themes";
import { PortRefStoreContext } from "../hooks/use-port-location";

const selector = (s: ReactFlowState) => ({
  nodeInternals: s.nodes,
  edges: s.edges,
  handle: s.connectionStartHandle,
});

type TargetPortProps = { id: `t#${number}`; label?: string; top: number };
function TargetPort(props: TargetPortProps) {
  const nodeId = useNodeId();
  const [portRef, setPortRef] = useState<HTMLDivElement | null>(null);
  const portRefStore = useContext(PortRefStoreContext);
  useEffect(() => {
    if (portRef) {
      const id = `${nodeId}#${props.id}`;
      portRefStore.add(id, portRef);
      return () => {
        portRefStore.remove(id);
      };
    }
  }, [portRef, portRefStore, props.id, nodeId]);
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
    const result =
      connections.length == 0 &&
      !!handlePortType &&
      !!portType &&
      WbblWebappGraphStore.are_port_types_compatible(portType, handlePortType);
    return result;
  }, [connections.length, portType, handlePortType]);

  const handleClassName = useMemo(
    () => `${getStyleForType(portType)} ${isHandleConnectable ? "glow" : " "}`,
    [isHandleConnectable, portType],
  );

  const handleStyle = useMemo(
    () => ({
      width: PORT_SIZE - 4,
      height: PORT_SIZE - 4,
      borderWidth: 2,
      left: PORT_SIZE,
      top: props.top,
      background: "transparent",
      position: "absolute" as const,
      transitionDuration: "300ms",
      transitionProperty: "stroke",
    }),
    [props.top],
  );

  const textStyle = useMemo(
    () => ({
      top: `calc(${props.top}px - 0.8em)`,
      left: 1.8 * PORT_SIZE,
      position: "absolute" as const,
      textAlign: "left" as const,
      fontSize: "0.8em",
    }),
    [props.top],
  );

  const text = useMemo(() => {
    return (
      props.label && (
        <Text
          className="port-label"
          key="label"
          as="label"
          htmlFor={props.id}
          style={textStyle}
        >
          {props.label}
        </Text>
      )
    );
  }, [textStyle, props.label]);

  const handleElement = useMemo(() => {
    return (
      <Handle
        type="target"
        key="handle"
        ref={setPortRef}
        id={props.id}
        position={Position.Left}
        style={handleStyle}
        isConnectableStart={false}
        className={handleClassName}
        isConnectable={isHandleConnectable}
      />
    );
  }, [props.id, handleStyle, handleClassName, isHandleConnectable, portRef]);

  return (
    <>
      {handleElement}
      {text}
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
