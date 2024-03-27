import { Handle, Position } from "@xyflow/react";
import usePortType from "../hooks/use-port-type";
import { getStyleForPortType } from "../port-type-styling";

export default function SourcePort(props: {
  id: `s#${number}`;
  label?: string;
}) {
  const portType = usePortType(props.id);

  return (
    <div className="inline-flex justify-end gap-0 pr-4">
      {props.label && (
        <div className="font-mono text-sm italic">{props.label}</div>
      )}
      <Handle
        type="source"
        id={props.id}
        position={Position.Right}
        style={{
          width: 15,
          height: 15,
          padding: 0,
          margin: 0,
        }}
        className={`relative ${getStyleForPortType(portType)}`}
        isConnectable={true}
        isConnectableStart={true}
        isConnectableEnd={false}
      />
    </div>
  );
}
