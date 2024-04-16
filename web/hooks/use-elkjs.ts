import ELK, { ElkNode, ElkPort, LayoutOptions } from "elkjs/lib/elk-api.js";
import { useCallback, useContext } from "react";
import {
  WbblGraphStoreContext,
  WbblWebappGraphSnapshot,
} from "./use-wbbl-graph-store";
import { type Node, type Edge, useStoreApi, InternalNode } from "@xyflow/react";
import { EdgeStyle } from "../../pkg/wbbl";
import { WbblPreferencesStoreContext } from "./use-preferences-store";

const elk = new ELK({
  algorithms: ["layered", "fixed"],
  workerFactory: function () {
    // the value of 'url' is irrelevant here
    return new Worker("/node_modules/elkjs/lib/elk-worker.js", {
      type: "module",
      credentials: "same-origin",
    });
  },
});

function getNode(internalNodes: Map<string, InternalNode>, node: Node) {
  const handleBounds = internalNodes.get(node.id)?.internals.handleBounds;
  return {
    id: node.id,
    width: node.measured?.width ?? 0,
    height: node.measured?.height ?? 0,
    layoutOptions: node.selected
      ? {}
      : ({
          "elk.algorithm": "fixed",
        } as { [key: string]: string }),
    x: node.position.x,
    y: node.position.y,
    ports: (handleBounds?.source ?? [])
      .map<ElkPort>(
        (x) =>
          ({
            ...x,
          }) as ElkPort,
      )
      .concat(
        (handleBounds?.target ?? []).map<ElkPort>(
          (x) =>
            ({
              ...x,
            }) as ElkPort,
        ),
      )
      .map((x) => {
        return { ...x, id: `${node.id}#${x.id}` };
      }) as ElkPort[],
  };
}

async function elkLayout(
  internalNodes: Map<string, InternalNode>,
  nodes: (Node & { groupId?: string })[],
  edges: Edge[],
  edgeStyle: EdgeStyle,
) {
  const nodeMap = new Set(nodes.map((x) => x.id));

  const layoutOptions = {
    "elk.algorithm": "layered",
    "elk.direction": "RIGHT",
    "elk.alignment": "TOP",
    "elk.port.anchor": `35`,
    "elk.portConstraints": "FIXED_POS",
    "elk.graphviz.adaptPortPositions": "false",
    "elk.edgeRouting":
      edgeStyle === EdgeStyle.Bezier
        ? "SPLINE"
        : edgeStyle === EdgeStyle.Metropolis
          ? "ORTHOGONAL"
          : "UNDEFINED",
    "elk.spacing.nodeNode": `50`,
    "elk.layered.considerModelOrder.portModelOrder": "true",
    "elk.hierarchyHandling": "INCLUDE_CHILDREN",
  };

  const children: Map<string, ElkNode[]> = new Map();
  const noGroup: ElkNode[] = [];
  for (const node of nodes) {
    if (node.groupId) {
      if (!children.has(node.groupId)) {
        children.set(node.groupId, []);
      }
      children.get(node.groupId)!.push(getNode(internalNodes, node));
    } else {
      noGroup.push(getNode(internalNodes, node));
    }
  }
  const graph: ElkNode = {
    id: "elk-root",
    layoutOptions,
    children: noGroup.concat(
      [...children.entries()].map(
        ([k, v]) =>
          ({
            id: k,
            children: [...v],
            layoutOptions: {
              ...layoutOptions,
            } as LayoutOptions,
          }) as ElkNode,
      ),
    ),
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

  const nextNodes = nodes
    .filter((x) => x.selected)
    .map((node) => {
      let position = { x: 0, y: 0 };
      if (node.groupId) {
        const groupNode = layoutNodes.get(node.groupId)!;
        const elkNode = groupNode.children!.find((x) => x.id === node.id)!;
        position = {
          x: elkNode.x! + groupNode.x!,
          y: elkNode.y! + groupNode.y!,
        };
      } else {
        const elkNode = layoutNodes.get(node.id)!;
        position = { x: elkNode.x!, y: elkNode.y! };
      }

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

  return useCallback(
    async (nodes?: Set<string>, edges?: Set<string>) => {
      const thisSnapshot = graphStore.get_snapshot() as WbblWebappGraphSnapshot;
      if (nodes !== undefined) {
        thisSnapshot.nodes.forEach((x) => {
          x.selected = nodes.has(x.id);
        });
      }
      if (edges !== undefined) {
        thisSnapshot.edges.forEach((x) => {
          x.selected = edges.has(x.id);
        });
      }

      const results = await elkLayout(
        storeApi.getState().nodeLookup,
        thisSnapshot.nodes,
        thisSnapshot.edges,
        preferencesStore.get_edge_style(),
      );
      for (const node of results.nodes) {
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
    },
    [graphStore, storeApi, preferencesStore],
  );
}
