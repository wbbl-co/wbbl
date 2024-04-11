import ELK, { ElkNode, ElkPort } from "elkjs/lib/elk-api.js";
import { useCallback, useContext } from "react";
import {
  WbblGraphStoreContext,
  WbblWebappGraphSnapshot,
} from "./use-wbbl-graph-store";
import { type Node, type Edge, useStoreApi, InternalNode } from "@xyflow/react";
import { HALF_PORT_SIZE, PORT_SIZE } from "../port-constants";
import { EdgeStyle } from "../../pkg/wbbl";
import { WbblPreferencesStoreContext } from "./use-preferences-store";

const elk = new ELK({
  algorithms: ["layered"],
  workerFactory: function (_) {
    // the value of 'url' is irrelevant here
    return new Worker("/node_modules/elkjs/lib/elk-worker.js", {
      type: "module",
      credentials: "same-origin",
    });
  },
});

async function elkLayout(
  internalNodes: Map<String, InternalNode>,
  nodes: Node[],
  edges: Edge[],
  edgeStyle: EdgeStyle,
) {
  const nodeMap = new Set(nodes.map((x) => x.id));

  const layoutOptions = {
    "elk.algorithm": "layered",
    "elk.direction": "RIGHT",
    "elk.alignment": "TOP",
    "elk.port.anchor": `35`,
    "elk.port.borderOffset": `${PORT_SIZE}`,
    "elk.portConstraints": "FIXED_ORDER",
    "elk.edgeRouting":
      edgeStyle === EdgeStyle.Bezier
        ? "SPLINE"
        : edgeStyle === EdgeStyle.Metropolis
          ? "ORTHOGONAL"
          : "UNDEFINED",
    "elk.spacing.nodeNode": `50`,
    "elk.spacing.portPort": `${PORT_SIZE + HALF_PORT_SIZE}`,
  };

  const graph: ElkNode = {
    id: "elk-root",
    layoutOptions,
    children: nodes.map((node) => {
      let handleBounds = internalNodes.get(node.id)?.internals.handleBounds;
      return {
        id: node.id,
        width: node.measured?.width ?? 0,
        height: node.measured?.height ?? 0,
        properties: {
          "org.eclipse.elk.portConstraints": "FIXED_ORDER",
        },
        ports: (handleBounds?.source ?? [])
          .map<ElkPort>(
            (x) => ({ ...x, properties: { side: "EAST", ...x } }) as ElkPort,
          )
          .concat(
            (handleBounds?.target ?? []).map<ElkPort>(
              (x) =>
                ({
                  ...x,
                  properties: { side: "WEST", ...x },
                }) as ElkPort,
            ),
          )
          .map((x) => {
            return { ...x, id: `${node.id}#${x.id}` };
          }) as ElkPort[],
      };
    }),
    edges: edges
      .filter((edge) => nodeMap.has(edge.source) && nodeMap.has(edge.target))
      .map((edge) => ({
        id: edge.id,
        sources: [`${edge.source}#${edge.sourceHandle}`],
        targets: [`${edge.target}#${edge.targetHandle}`],
      })),
  };

  // We create a map of the laid out nodes here to avoid multiple traversals when
  // looking up a node's position later on.
  const root = await elk.layout(graph, {
    layoutOptions,
  });

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
  const preferencesStore = useContext(WbblPreferencesStoreContext);

  return useCallback(async () => {
    let thisSnapshot = graphStore.get_snapshot() as WbblWebappGraphSnapshot;
    thisSnapshot.nodes = thisSnapshot.nodes.filter((x) => x.selected);
    const results = await elkLayout(
      storeApi.getState().nodeLookup,
      thisSnapshot.nodes,
      thisSnapshot.edges,
      preferencesStore.get_edge_style(),
    );
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
        duration: 1000,
        minZoom: zoom,
        maxZoom: zoom,
      });
    }, 30);
  }, [graphStore, storeApi, preferencesStore]);
}
