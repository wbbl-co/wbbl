import { Handle, Position } from "@xyflow/react";

export default function SourcePort(props: { id: string; label?: string }) {
  return (
    <div className="inline-flex min-w-12 justify-end gap-0 pr-6">
      {props.label && (
        <div className="text-md font-mono italic">{props.label}</div>
      )}
      <Handle
        type="source"
        id={props.id}
        position={Position.Right}
        style={{
          width: 20,
          height: 20,
        }}
        className="bg-lime relative"
        isConnectable={true}
      />
    </div>
  );
}
