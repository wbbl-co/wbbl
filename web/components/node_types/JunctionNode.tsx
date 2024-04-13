import { NodeProps, useHandleConnections } from "@xyflow/react";
import { memo } from "react";
import { areNodePropsEqual } from "../../hooks/use-wbbl-graph-store";
import NodeContextMenu from "../NodeOrEdgeContextMenu";
import { ShortcutScope } from "../../hooks/use-shortcut";
import { Box, Card, Flex, Separator } from "@radix-ui/themes";
import { nodeMetaData } from ".";
import { HALF_PORT_SIZE, PORT_SIZE } from "../../port-constants";
import SourcePort from "../SourcePort";
import TargetPort from "../TargetPort";
import { useCardWbbl } from "../../hooks/use-card-wbbl";

export const JUNCTION_WIDTH = PORT_SIZE * 5;
export const JUNCTION_HEIGHT = PORT_SIZE * 3;
function JunctionNode({
  id,
  type,
  selected,
  positionAbsoluteX,
  positionAbsoluteY,
  dragging,
}: Omit<NodeProps, "width" | "height">) {
  const contentsRef = useCardWbbl({
    w: JUNCTION_WIDTH,
    h: JUNCTION_HEIGHT,
    positionAbsoluteX,
    positionAbsoluteY,
    dragging,
    selected: !!selected,
  });
  const sourceEdges = useHandleConnections({
    id: "s#0",
    nodeId: id,
    type: "source",
  });
  const targetEdges = useHandleConnections({
    id: "t#0",
    nodeId: id,
    type: "target",
  });

  return (
    <ShortcutScope scope={`node-${id}`} mode="hover">
      <NodeContextMenu
        isEdge={false}
        selected={selected ?? false}
        previewable={true}
        deleteable={true}
        copyable={true}
        id={id}
        type={type}
      >
        <Box
          style={{
            width: JUNCTION_WIDTH,
            height: JUNCTION_HEIGHT,
            overflow: "visible",
            padding: 0,
            margin: 0,
          }}
        >
          <Card
            ref={contentsRef}
            aria-selected={selected}
            data-connected={
              sourceEdges.length > 0 && targetEdges.length > 0
                ? "true"
                : "false"
            }
            className={`node-contents junction ${selected ? "selected" : ""} category-${nodeMetaData[type as keyof typeof nodeMetaData].category}`}
            style={{
              width: JUNCTION_WIDTH,
              height: JUNCTION_HEIGHT,
            }}
          >
            <Flex
              style={{
                width: JUNCTION_WIDTH,
                height: JUNCTION_HEIGHT,
                padding: 0,
                margin: 0,
              }}
              justify={"center"}
              align={"center"}
            >
              <Separator
                decorative
                style={{ borderWidth: "1em", height: "100%", width: 2 }}
                className={`junction-separator category-${nodeMetaData[type as keyof typeof nodeMetaData].category}`}
                orientation={"vertical"}
                my="3"
                size="4"
              />
            </Flex>
            <SourcePort id={"s#0"} top={PORT_SIZE + HALF_PORT_SIZE} />
            <TargetPort id={"t#0"} top={PORT_SIZE + HALF_PORT_SIZE} />
          </Card>
        </Box>
      </NodeContextMenu>
    </ShortcutScope>
  );
}

export default memo(JunctionNode, areNodePropsEqual);
