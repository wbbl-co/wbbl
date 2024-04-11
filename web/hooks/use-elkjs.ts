import ELK, { type ELK as ElkType, ElkNode } from "elkjs/lib/elk-api.js";
import { useCallback, useContext } from "react";
import {
  WbblGraphStoreContext,
  WbblWebappGraphSnapshot,
} from "./use-wbbl-graph-store";
import { type Node, type Edge, useStoreApi } from "@xyflow/react";

export type LayoutAlgorithm = (
  elk: ElkType,
  nodes: Node[],
  edges: Edge[],
) => Promise<{ nodes: Node[]; edges: Edge[] }>;

const elk = new ELK({
  workerFactory: function (_) {
    // the value of 'url' is irrelevant here
    return new Worker("/node_modules/elkjs/lib/elk-worker.js", {
      type: "module",
      credentials: "same-origin",
    });
  },
});

async function elkLayout(nodes: Node[], edges: Edge[]) {
  const graph = {
    id: "elk-root",
    layoutOptions: {
      "elk.algorithm": "layered",
      "elk.direction": "RIGHT",
      "elk.spacing.nodeNode": `50`,
    },
    children: nodes.map((node) => ({
      id: node.id,
      width: node.measured?.width ?? 0,
      height: node.measured?.height ?? 0,
    })),
    edges: edges.map((edge) => ({
      id: edge.id,
      sources: [edge.source],
      targets: [edge.target],
    })),
  };

  // We create a map of the laid out nodes here to avoid multiple traversals when
  // looking up a node's position later on.
  const root = await elk.layout(graph);
  const layoutNodes = new Map<string, ElkNode>();
  for (const node of root.children ?? []) {
    layoutNodes.set(node.id, node);
  }

  const nextNodes = nodes.map((node) => {
    const elkNode = layoutNodes.get(node.id)!;
    const position = { x: elkNode.x!, y: elkNode.y! };

    return {
      id: node.id,
      position,
    };
  });

  return { nodes: nextNodes };
}

export function useElkJs() {
  const graphStore = useContext(WbblGraphStoreContext);
  const storeApi = useStoreApi();
  return useCallback(async () => {
    let thisSnapshot = graphStore.get_snapshot() as WbblWebappGraphSnapshot;
    thisSnapshot.nodes = thisSnapshot.nodes.filter((x) => x.selected);
    const results = await elkLayout(thisSnapshot.nodes, thisSnapshot.edges);
    for (let node of results.nodes) {
      graphStore.set_node_position(
        node.id,
        node.position.x,
        node.position.y,
        false,
      );
    }
    setTimeout(() => {
      const state = storeApi.getState();
      const zoom = state.panZoom?.getViewport().zoom;
      state.fitView({
        nodes: results.nodes,
        duration: 0,
        minZoom: zoom,
        maxZoom: zoom,
      });
    }, 30);
  }, [graphStore, storeApi]);
}
