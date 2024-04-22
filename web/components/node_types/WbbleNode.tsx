import { NodeProps } from "@xyflow/react";
import { ReactElement, memo, useMemo } from "react";
import TargetPort from "../TargetPort";
import SourcePort from "../SourcePort";
import { HALF_PORT_SIZE, PORT_SIZE } from "../../port-constants";
import { nodeMetaData } from ".";
import NodeContextMenu from "../NodeOrEdgeContextMenu";
import { Card, Heading, Flex } from "@radix-ui/themes";
import { Box } from "@radix-ui/themes";
import { ShortcutScope } from "../../hooks/use-shortcut";
import { useCardWbbl } from "../../hooks/use-card-wbbl";

function WbblNode({
  id,
  type,
  dragging,
  positionAbsoluteX,
  positionAbsoluteY,
  width,
  height,
  children,
  inputPortLabels,
  outputPortLabels,
  previewable,
  deleteable,
  copyable,
  selected,
}: Omit<NodeProps, "data"> & {
  inputPortLabels: (null | string)[];
  outputPortLabels: (null | string)[];
  children?: ReactElement;
  previewable: boolean;
  deleteable: boolean;
  copyable: boolean;
}) {
  const contentsRef = useCardWbbl({
    w: width!,
    h: height!,
    positionAbsoluteX,
    positionAbsoluteY,
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
    <ShortcutScope scope={`node-${id}`} mode="hover">
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

export default memo(WbblNode);
