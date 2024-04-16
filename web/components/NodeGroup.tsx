import { MouseEventHandler, useCallback, useContext, useMemo } from "react";
import { WbblGraphStoreContext } from "../hooks/use-wbbl-graph-store";
import { ShortcutScope } from "../hooks/use-shortcut";
import GroupContextMenu from "./GroupContextMenu";

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

export default function NodeGroup(props: {
  id: string;
  path?: string;
  nodes: string[];
  edges: string[];
}) {
  const graphStore = useContext(WbblGraphStoreContext);

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
              style={{ stroke: `var(--${color}-a4)`, strokeWidth: 1 }}
            />
          </pattern>
        </defs>
        <path
          onClick={onClick}
          strokeWidth={"2"}
          fill={`url(#diagonalHatch-${props.id})`}
          stroke={`var(--${color}-9)`}
          style={{ pointerEvents: "visible" }}
          d={props.path ?? ""}
        />
      </GroupContextMenu>
    </ShortcutScope>
  );
}
