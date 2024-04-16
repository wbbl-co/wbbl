import { ViewportPortal } from "@xyflow/react";
import NodeGroup from "./NodeGroup";

type NodeGroupRendererProps = {
  groups: {
    id: string;
    path?: string;
    nodes: string[];
    edges: string[];
    bounds: Float32Array;
  }[];
  width: number;
  height: number;
};

export function NodeGroupRenderer({
  groups,
  width,
  height,
}: NodeGroupRendererProps) {
  return (
    <ViewportPortal>
      <svg
        id="group-renderer"
        style={{
          width: width,
          overflow: "visible",
          height: height,
          position: "absolute",
          pointerEvents: "none",
          left: 0,
          top: 0,
          transformOrigin: "0 0",
        }}
      >
        {groups.map((g) => {
          return (
            <NodeGroup
              key={g.id}
              id={g.id}
              path={g.path}
              nodes={g.nodes}
              edges={g.edges}
              bounds={g.bounds}
            />
          );
        })}
      </svg>
    </ViewportPortal>
  );
}
