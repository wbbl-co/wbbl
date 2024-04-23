import { NodeProps, ReactFlowStore, useStore } from "@xyflow/react";
import { ReactElement, memo, useCallback, useMemo } from "react";
import TargetPort from "../TargetPort";
import SourcePort from "../SourcePort";
import { HALF_PORT_SIZE, PORT_SIZE } from "../../port-constants";
import { nodeMetaData } from ".";
import NodeContextMenu from "../NodeOrEdgeContextMenu";
import { Card, Heading, Flex } from "@radix-ui/themes";
import { Box } from "@radix-ui/themes";
import { ShortcutScope } from "../../hooks/use-shortcut";
import { useCardWbbl } from "../../hooks/use-card-wbbl";

function positionSelector(id: string) {
  return (store: ReactFlowStore) => store.nodeLookup.get(id)!.position;
}

function WbblNode({
  id,
  type,
  dragging,
  width,
  height,
  children,
  inputPortLabels,
  outputPortLabels,
  previewable,
  deleteable,
  copyable,
  selected,
  positionX,
  positionY,
}: Omit<NodeProps, "data" | "positionAbsoluteX" | "positionAbsoluteY"> & {
  inputPortLabels: (null | string)[];
  outputPortLabels: (null | string)[];
  children?: ReactElement;
  previewable: boolean;
  deleteable: boolean;
  copyable: boolean;
  positionX: number;
  positionY: number;
}) {
  const contentsRef = useCardWbbl({
    w: width!,
    h: height!,
    positionAbsoluteX: positionX,
    positionAbsoluteY: positionY,
    dragging,
    selected: !!selected,
  });
  const outputPorts = useMemo(
    () =>
      outputPortLabels.map((x: string | null, idx: number) => (
        <SourcePort
          top={idx * (PORT_SIZE + HALF_PORT_SIZE) + 35}
          id={`s#${idx}`}
          label={x ?? undefined}
          key={idx}
        />
      )),
    [outputPortLabels],
  );

  const inputPorts = useMemo(
    () =>
      inputPortLabels.map((x: string | null, idx: number) => (
        <TargetPort
          top={idx * (PORT_SIZE + HALF_PORT_SIZE) + 35}
          id={`t#${idx}`}
          label={x ?? undefined}
          key={idx}
        />
      )),
    [inputPortLabels],
  );

  const contents = useMemo(
    () => (
      <Box
        style={{
          width: width!,
          height: height!,
          overflow: "visible",
          padding: 0,
          margin: 0,
        }}
      >
        <Card
          ref={contentsRef}
          aria-selected={selected}
          className={`node-contents ${selected ? "selected" : ""} category-${nodeMetaData[type as keyof typeof nodeMetaData].category}`}
          style={{
            width: width!,
            height: height!,
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
          {inputPorts}
          {outputPorts}
        </Card>
      </Box>
    ),
    [selected, outputPorts, inputPorts, children, type, contentsRef],
  );

  return (
    <ShortcutScope
      style={{ width: width, height: height }}
      scope={`node-${id}`}
      mode="hover"
    >
      <NodeContextMenu
        isEdge={false}
        selected={selected ?? false}
        previewable={previewable}
        deleteable={deleteable}
        copyable={copyable}
        id={id}
        type={type}
      >
        {contents}
      </NodeContextMenu>
    </ShortcutScope>
  );
}

const MemoWbblNode = memo(WbblNode);

export type WbblNodeProps = Omit<
  NodeProps,
  "data" | "positionAbsoluteX" | "positionAbsoluteY"
> & {
  inputPortLabels: (null | string)[];
  outputPortLabels: (null | string)[];
  children?: ReactElement;
  previewable: boolean;
  deleteable: boolean;
  copyable: boolean;
};

function WbblNodeMemoWrapper(props: WbblNodeProps) {
  const positionSelectorForId = useCallback(positionSelector(props.id), [
    props.id,
  ]);
  const position = useStore(positionSelectorForId)!;
  return (
    <MemoWbblNode positionX={position.x} positionY={position.y} {...props} />
  );
}

const shallowProps = [
  "children",
  "previewable",
  "deleteable",
  "selected",
  "copyable",
  "dragging",
  "width",
  "height",
] as const;

export function areWbblNodePropsEqual(
  oldProps: WbblNodeProps,
  newProps: WbblNodeProps,
) {
  for (const prop of shallowProps) {
    if (oldProps[prop] !== newProps[prop]) {
      return false;
    }
  }

  if (
    oldProps.inputPortLabels === newProps.inputPortLabels &&
    oldProps.outputPortLabels === newProps.outputPortLabels
  ) {
    return true;
  }

  if (
    oldProps.inputPortLabels.length !== newProps.inputPortLabels.length ||
    oldProps.outputPortLabels.length !== newProps.outputPortLabels.length
  ) {
    return false;
  }

  return (
    oldProps.inputPortLabels.every(
      (x, i) => x === newProps.inputPortLabels[i],
    ) &&
    oldProps.outputPortLabels.every(
      (x, i) => x === newProps.outputPortLabels[i],
    )
  );
}

export default memo(WbblNodeMemoWrapper, areWbblNodePropsEqual);
