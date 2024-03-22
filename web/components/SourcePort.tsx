import { Handle, Position } from "@xyflow/react";

export default function SourcePort(props: {
  id: `s-${number}`;
  label?: string;
}) {
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
        className="bg-lime relative"
        isConnectable={true}
        isConnectableStart={true}
        isConnectableEnd={false}
      />
    </div>
  );
}
