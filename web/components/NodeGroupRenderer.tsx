import { useViewport } from "@xyflow/react";

type NodeGroupRendererProps = {
  groups: { id: string; path?: string }[];
  width: number;
  height: number;
};
export function NodeGroupRenderer({
  groups,
  width,
  height,
}: NodeGroupRendererProps) {
  const viewport = useViewport();
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
        zIndex: -1,
        transform: `translate(${viewport.x}px, ${viewport.y}px) scale(${viewport.zoom})`,
      }}
    >
      <defs>
        <pattern
          id="diagonalHatch"
          patternUnits="userSpaceOnUse"
          width="8"
          height="8"
          stroke="currentcolor"
        >
          <line
            x1="0"
            y1="0"
            x2="8"
            y2="8"
            style={{ stroke: "var(--lime-a4)", strokeWidth: 1 }}
          />
        </pattern>
      </defs>
      {groups.map((g) => {
        return (
          <path
            key={g.path ?? ""}
            strokeWidth={"2"}
            fill="url(#diagonalHatch)"
            stroke="var(--lime-9)"
            d={g.path ?? ""}
          />
        );
      })}
    </svg>
  );
}
