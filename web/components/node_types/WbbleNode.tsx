import { NodeProps } from "@xyflow/react";
import {
  ReactElement,
  MouseEvent,
  useCallback,
  useEffect,
  useRef,
  useState,
} from "react";
import { WbblBox } from "../../../pkg/wbbl";
import TargetPort from "../TargetPort";
import SourcePort from "../SourcePort";
import { HALF_PORT_SIZE, PORT_SIZE } from "../../port-constants";
import { nodeMetaData } from ".";
import { NodeContextMenu } from "../NodeContextMenu";
import { Card, Heading, Flex } from "@radix-ui/themes";
import { Box } from "@radix-ui/themes/dist/cjs/index.js";

function WbblNode({
  id,
  type,
  dragging,
  positionAbsoluteX,
  positionAbsoluteY,
  w,
  h,
  children,
  inputPortLabels,
  outputPortLabels,
  previewable,
  deleteable,
  copyable,
  selected,
}: Omit<NodeProps, "width" | "height"> & {
  inputPortLabels: (null | string)[];
  outputPortLabels: (null | string)[];
  w: number;
  h: number;
  children: ReactElement;
  previewable: boolean;
  deleteable: boolean;
  copyable: boolean;
}) {
  const [box] = useState(() =>
    WbblBox.new(
      new Float32Array([positionAbsoluteX + h / 2, positionAbsoluteY + h / 2]),
      new Float32Array([w, h]),
    ),
  );
  const [dragOrigin, setDragOrigin] = useState<[number, number]>([
    w / 2,
    h / 2,
  ]);
  const contentsRef = useRef<HTMLDivElement>(null);
  const lastUpdate = useRef<number>(Date.now());

  useEffect(() => {
    let animationFrame: number;
    function update(time: DOMHighResTimeStamp) {
      const delta = Math.min(
        0.25,
        Math.max(0.0, (time - lastUpdate.current) / 1000.0),
      );
      box.update(
        new Float32Array([positionAbsoluteX, positionAbsoluteY]),
        new Float32Array([w, h]),
        delta,
        dragging
          ? new Float32Array([
              positionAbsoluteX + dragOrigin[0],
              positionAbsoluteY + dragOrigin[1],
            ])
          : undefined,
      );

      if (contentsRef.current) {
        let skew = box.get_skew(new Float32Array([w, h]));
        contentsRef.current.style.transform = skew;
      }

      lastUpdate.current = time;
      animationFrame = requestAnimationFrame(update);
    }
    update(lastUpdate.current);
    return () => cancelAnimationFrame(animationFrame);
  }, [
    box,
    contentsRef,
    lastUpdate,
    dragging,
    positionAbsoluteX,
    positionAbsoluteY,
    w,
    h,
    type,
    dragOrigin,
  ]);

  const onDrag = useCallback(
    (evt: MouseEvent<HTMLDivElement>) => {
      let rect = (evt.target as HTMLDivElement).getBoundingClientRect();
      setDragOrigin([evt.screenX - rect.x, evt.screenY - rect.y]);
    },
    [setDragOrigin],
  );

  return (
    <NodeContextMenu
      previewable={previewable}
      deleteable={deleteable}
      copyable={copyable}
      id={id}
      type={type}
    >
      <Box
        style={{
          width: w,
          height: h,
          overflow: "visible",
          padding: 0,
          margin: 0,
        }}
      >
        <Card
          onDragStartCapture={onDrag}
          ref={contentsRef}
          aria-selected={selected}
          className={`node-contents ${selected ? "selected" : ""} category-${nodeMetaData[type as keyof typeof nodeMetaData].category}`}
          style={{
            width: w,
            height: h,
            color: `var(--${nodeMetaData[type as keyof typeof nodeMetaData].category}-color)`,
          }}
        >
          <Heading
            as="h3"
            align="center"
            size={"4"}
            className="node-type-heading"
          >
            {type}
          </Heading>
          <Flex justify={"center"} align={"center"}>
            {children}
          </Flex>
          {inputPortLabels.map((x: string | null, idx: number) => (
            <TargetPort
              top={idx * (PORT_SIZE + HALF_PORT_SIZE) + 35}
              id={`t#${idx}`}
              label={x ?? undefined}
              key={idx}
            />
          ))}
          {outputPortLabels.map((x: string | null, idx: number) => (
            <SourcePort
              top={idx * (PORT_SIZE + HALF_PORT_SIZE) + 35}
              id={`s#${idx}`}
              label={x ?? undefined}
              key={idx}
            />
          ))}
        </Card>
      </Box>
    </NodeContextMenu>
  );
}

export default WbblNode;
