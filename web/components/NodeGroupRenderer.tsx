import { useViewport } from "@xyflow/react";
import { useContext } from "react";
import { WbblGraphStoreContext } from "../hooks/use-wbbl-graph-store";

type NodeGroupRendererProps = {
  groups: { id: string }[];
  width: number;
  height: number;
};
export function NodeGroupRenderer({
  groups,
  width,
  height,
}: NodeGroupRendererProps) {
  const viewport = useViewport();
  const graphStore = useContext(WbblGraphStoreContext);
  return (
    <svg
      id="group-renderer"
      style={{
        width: width,
        overflow: "visible",
        height: height,
        pointerEvents: "none",
        position: "absolute",
        left: 0,
        top: 0,
        transformOrigin: "0 0",
        transform: `translate(${viewport.x}px, ${viewport.y}px) scale(${viewport.zoom})`,
      }}
    >
      {groups.map((g) => {
        let bounds = graphStore.get_group_bounds(g.id);
        let path: string = `M ${bounds[0]} ${bounds[1]}`;
        for (let i = 2; i < bounds.length - 1; i += 2) {
          path += ` L ${bounds[i]} ${bounds[i + 1]}`;
        }
        path += ` Z`;

        return <path key={g.id} fill="red" stroke="blue" d={path} />;
      })}
    </svg>
  );
}
