import {
  MouseEventHandler,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
} from "react";
import { WbblGraphStoreContext } from "../hooks/use-wbbl-graph-store";
import { ShortcutScope } from "../hooks/use-shortcut";
import GroupContextMenu from "./GroupContextMenu";
import { ReactFlowStore, useReactFlow, useStore } from "@xyflow/react";
import { is_axis_aligned_rect_intersecting_convex_hull } from "../../pkg/wbbl";

const colors = [
  "red",
  "yellow",
  "blue",
  "lime",
  "orange",
  "violet",
  "green",
  "gray",
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
  bounds: Float32Array;
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
          props.bounds,
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

  const onClick = useCallback<MouseEventHandler<SVGPathElement>>(() => {
    graphStore.select_group(props.id, true);
  }, [graphStore]);
  const color = useMemo(() => {
    return uuidToColor(props.id);
  }, [props.id]);

  return (
    <ShortcutScope scope={`group-${props.id}`} as="g" mode="hover">
      <GroupContextMenu
        nodes={props.nodes}
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
          onClick={onClick}
          strokeWidth={"2"}
          fill={`url(#diagonalHatch-${props.id})`}
          stroke={`var(--${color}-9)`}
          style={{ pointerEvents: selectionRect ? "none" : "visible" }}
          d={props.path ?? ""}
        />
      </GroupContextMenu>
    </ShortcutScope>
  );
}
