import { useCallback, useContext, useEffect, useMemo, useRef } from "react";
import { WbblGraphStoreContext } from "../hooks/use-wbbl-graph-store";
import { ShortcutScope } from "../hooks/use-shortcut";
import GroupContextMenu from "./GroupContextMenu";
import {
  ReactFlowStore,
  useReactFlow,
  useStore,
  useStoreApi,
} from "@xyflow/react";
import { is_axis_aligned_rect_intersecting_convex_hull } from "../../pkg/wbbl";
import { useGroupDrag } from "../hooks/use-group-drag";
import { handleGroupClick } from "../hooks/use-group-drag/utils";

const colors = [
  "red",
  "yellow",
  "blue",
  "lime",
  "orange",
  "violet",
  "green",
] as const;

function uuidToColor(id: string) {
  let hash = 0;
  id.split("").forEach((char) => {
    hash = char.charCodeAt(0) + ((hash << 5) - hash);
  });

  return colors[Math.abs(hash) % colors.length];
}

function selector(store: ReactFlowStore) {
  return store.userSelectionRect;
}
export default function NodeGroup(props: {
  id: string;
  path?: string;
  nodes: string[];
  edges: string[];
  selected: boolean;
  bounds: [number, number][];
}) {
  const graphStore = useContext(WbblGraphStoreContext);
  const selectionRect = useStore(selector);
  const flow = useReactFlow();
  useEffect(() => {
    if (selectionRect) {
      const topLeft = flow.screenToFlowPosition(
        {
          x: selectionRect.x,
          y: selectionRect.y,
        },
        { snapToGrid: false },
      );
      const bottomRight = flow.screenToFlowPosition(
        {
          x: selectionRect.x + selectionRect.width,
          y: selectionRect.y + selectionRect.height,
        },
        { snapToGrid: false },
      );
      if (
        is_axis_aligned_rect_intersecting_convex_hull(
          new Float32Array(props.bounds.flatMap((x) => x)),
          new Float32Array([topLeft.x, topLeft.y]),
          new Float32Array([bottomRight.x, bottomRight.y]),
        )
      ) {
        graphStore.select_group(props.id, true);
      } else {
        graphStore.deselect_group(props.id);
      }
    }
  }, [selectionRect, graphStore, props.id, flow]);

  const groupRef = useRef<SVGPathElement>(null);
  useGroupDrag({
    groupRef,
    groupId: props.id,
    disabled: false,
    isSelectable: true,
    selected: props.selected,
  });
  const color = useMemo(() => {
    return uuidToColor(props.id);
  }, [props.id]);
  const store = useStoreApi();

  const onDrag = useCallback(() => {
    handleGroupClick({
      groupId: props.id,
      store,
      groupRef,
      selected: props.selected,
      graphStore,
    });
  }, [props.id, store, props.selected, graphStore]);

  return (
    <ShortcutScope scope={`group-${props.id}`} as="g" mode="hover">
      <GroupContextMenu
        nodes={props.nodes}
        selected={props.selected}
        edges={props.edges}
        id={props.id}
        color={color}
      >
        <defs>
          <pattern
            id={`diagonalHatch-${props.id}`}
            patternUnits="userSpaceOnUse"
            width="8"
            height="8"
          >
            <line
              x1="0"
              y1="0"
              x2="8"
              y2="8"
              style={{
                stroke: `var(--${color}-a4)`,
                strokeWidth: 1,
              }}
            />
          </pattern>
        </defs>
        <path
          ref={groupRef}
          onPointerDown={onDrag}
          data-selected={`${props.selected}`}
          className="node-group"
          strokeWidth={"2"}
          fill={`url(#diagonalHatch-${props.id})`}
          stroke={`var(--${color}-9)`}
          style={{
            color: `var(--${color}-9)`,
            pointerEvents: selectionRect ? "none" : "visible",
          }}
          d={props.path ?? ""}
        />
      </GroupContextMenu>
    </ShortcutScope>
  );
}
