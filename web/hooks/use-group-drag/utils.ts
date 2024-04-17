import type { RefObject } from "react";
import type { StoreApi } from "zustand";

import type { ReactFlowState } from "@xyflow/react";
import { WbblWebappGraphStore } from "../../../pkg/wbbl";

// this handler is called by
// 1. the click handler when node is not draggable or selectNodesOnDrag = false
// or
// 2. the on drag start handler when node is draggable and selectNodesOnDrag = true
export function handleGroupClick({
  groupId,
  store,
  unselect = false,
  groupRef,
  graphStore,
  selected,
}: {
  groupId: string;
  store: {
    getState: StoreApi<ReactFlowState>["getState"];
    setState: StoreApi<ReactFlowState>["setState"];
  };
  graphStore: WbblWebappGraphStore;
  unselect?: boolean;
  groupRef?: RefObject<SVGPathElement>;
  selected: boolean;
}) {
  const { multiSelectionActive } = store.getState();
  store.setState({ nodesSelectionActive: false });

  if (!selected) {
    graphStore.select_group(groupId, multiSelectionActive);
  } else if (unselect || (selected && multiSelectionActive)) {
    graphStore.deselect_group(groupId);
    requestAnimationFrame(() => groupRef?.current?.blur());
  }
}
