import { Handle, Position, useNodeId } from "@xyflow/react";
import usePortType from "../hooks/use-port-type";
import { getStyleForType } from "../port-type-styling";
import { memo, useContext, useEffect, useState } from "react";
import { PORT_SIZE } from "../port-constants";
import { Text } from "@radix-ui/themes";
import { PortRefStoreContext } from "../hooks/use-port-location";

type SourcePortProps = { id: `s#${number}`; label?: string; top: number };

function SourcePort(props: SourcePortProps) {
  const portType = usePortType(props.id);
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
  return (
    <>
      {props.label && (
        <Text
          className="port-label"
          key="label"
          as="label"
          htmlFor={props.id}
          style={{
            top: props.top,
            right: 2 * PORT_SIZE,
            position: "absolute",
          }}
        >
          {props.label}
        </Text>
      )}
      <Handle
        type="source"
        key="handle"
        id={props.id}
        ref={setPortRef}
        position={Position.Right}
        style={{
          right: PORT_SIZE,
          width: PORT_SIZE,
          height: PORT_SIZE,
          top: props.top,
          position: "absolute",
          border: "none",
          transitionProperty: "background-color",
          transitionDuration: "300ms",
          filter: "drop-shadow(3px 5px 2px rgb(0 0 0 / 0.4))",
        }}
        className={`${getStyleForType(portType)}`}
        isConnectable={true}
        isConnectableStart={true}
        isConnectableEnd={false}
      />
    </>
  );
}

function propsAreEqual(
  oldProps: SourcePortProps,
  newProps: SourcePortProps,
): boolean {
  return (
    oldProps.id === newProps.id &&
    newProps.label === oldProps.label &&
    newProps.top == oldProps.top
  );
}

export default memo(SourcePort, propsAreEqual);
