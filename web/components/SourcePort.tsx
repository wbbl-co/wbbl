import { Handle, Position } from "@xyflow/react";
import usePortType from "../hooks/use-port-type";
import { getStyleForType } from "../port-type-styling";
import { memo } from "react";

type SourcePortProps = { id: `s#${number}`; label?: string; top: number };

function SourcePort(props: SourcePortProps) {
  const portType = usePortType(props.id);

  return (
    <>
      {props.label && (
        <div
          style={{ top: props.top - 10, right: 30 }}
          className="absolute font-mono text-sm italic"
        >
          {props.label}
        </div>
      )}
      <Handle
        type="source"
        id={props.id}
        position={Position.Right}
        style={{
          right: 15,
          width: 15,
          height: 15,
          top: props.top,
        }}
        className={`absolute border-none ${getStyleForType(portType)}`}
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
