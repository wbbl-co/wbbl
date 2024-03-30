import { Handle, Position } from "@xyflow/react";
import usePortType from "../hooks/use-port-type";
import { getStyleForType } from "../port-type-styling";
import { ForwardedRef, forwardRef, memo } from "react";
import { PORT_SIZE } from "../port-constants";
import { Text } from "@radix-ui/themes";

type SourcePortProps = { id: `s#${number}`; label?: string; top: number, width: number };

function SourcePort(props: SourcePortProps, forwardRef: ForwardedRef<HTMLDivElement>) {
  const portType = usePortType(props.id);

  return (
    <>
      {props.label && (
        <Text
          className="port-label"
          key="label"
          as="label"
          style={{ top: props.top - 10, right: 2 * PORT_SIZE, position: "absolute" }}
        >
          {props.label}
        </Text>
      )}
      <Handle
        type="source"
        key="handle"
        id={props.id}
        ref={forwardRef}
        position={Position.Right}
        style={{
          right: PORT_SIZE,
          width: PORT_SIZE,
          height: PORT_SIZE,
          transformOrigin: `${-props.top}px ${-(props.width - PORT_SIZE)}px`,
          top: props.top,
          position: "absolute",
          border: "none",
          transitionProperty: "background-color",
          transitionDuration: "300ms"
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

export default memo(forwardRef(SourcePort), propsAreEqual);
