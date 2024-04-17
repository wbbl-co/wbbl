import { useContext, useEffect, useRef, useState, type RefObject } from "react";
import { XYDrag, type XYDragInstance } from "@xyflow/system";

import { useStoreApi } from "@xyflow/react";
import { WbblGraphStoreContext } from "../use-wbbl-graph-store";

type UseDragParams = {
  groupRef: RefObject<SVGPathElement>;
  disabled?: boolean;
  noDragClassName?: string;
  handleSelector?: string;
  groupId?: string;
  isSelectable?: boolean;
  selected: boolean;
};

/**
 * Hook for calling XYDrag helper from @xyflow/system.
 *
 * @internal
 */
export function useGroupDrag({
  groupRef,
  disabled = false,
  noDragClassName,
  handleSelector,
  groupId,
  isSelectable,
  selected,
}: UseDragParams) {
  const graphStore = useContext(WbblGraphStoreContext);
  const store = useStoreApi();
  const [dragging, setDragging] = useState<boolean>(false);
  const xyDrag = useRef<XYDragInstance>();

  useEffect(() => {
    xyDrag.current = XYDrag({
      getStoreItems: () => store.getState(),
      onDragStart: () => {
        setDragging(true);
      },
      onDragStop: () => {
        setDragging(false);
      },
    });
  }, [graphStore, selected]);

  useEffect(() => {
    if (disabled) {
      xyDrag.current?.destroy();
    } else if (groupRef.current) {
      xyDrag.current?.update({
        noDragClassName,
        handleSelector,
        domNode: groupRef.current,
        isSelectable,
      });
      return () => {
        xyDrag.current?.destroy();
      };
    }
  }, [
    noDragClassName,
    handleSelector,
    disabled,
    isSelectable,
    groupRef,
    groupId,
  ]);

  return dragging;
}
