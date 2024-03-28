import { Handle, Position } from "@xyflow/react";
import usePortType from "../hooks/use-port-type";
import { getStyleForType } from "../port-type-styling";
import { memo } from "react";
import { HALF_PORT_SIZE, PORT_SIZE } from "../port-constants";

type SourcePortProps = { id: `s#${number}`; label?: string; top: number };

function SourcePort(props: SourcePortProps) {
  const portType = usePortType(props.id);

  return (
    <>
      {props.label && (
        <div
          key="label"
          style={{ top: props.top - 10, right: 2 * PORT_SIZE }}
          className="absolute font-mono text-sm italic"
        >
          {props.label}
        </div>
      )}
      <Handle
        type="source"
        key="handle"
        id={props.id}
        position={Position.Right}
        style={{
          right: PORT_SIZE,
          width: PORT_SIZE,
          height: PORT_SIZE,
          top: props.top,
        }}
        className={`absolute border-none transition-colors duration-300 ${getStyleForType(portType)}`}
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
