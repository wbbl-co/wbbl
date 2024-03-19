import { useCallback, useContext, useEffect, useRef } from "react";
import { useSyncExternalStore } from "react";
import { createContext } from "react";
import { WbblWebappGraphStore } from "../../pkg/wbbl";
import { NodeProps, Node, Edge } from "@xyflow/react";

export const WbblGraphStoreContext = createContext<WbblWebappGraphStore>(
  WbblWebappGraphStore.empty(),
);

export function useWbblGraphData(store: WbblWebappGraphStore): {
  edges: Edge[];
  nodes: Node[];
} {
  let data = useRef<{ edges: Edge[]; nodes: Node[] }>();
  let setup = useRef<boolean>(false);
  let subscribe = useCallback(
    (subscriber: () => void) => {
      if (!setup.current) {
        setup.current = true;
        store.subscribe(() => {
          data.current = undefined;
        });
      }
      let handle = store.subscribe(subscriber);
      return () => store.unsubscribe(handle);
    },
    [store],
  );

  let getSnapshot = useCallback(() => {
    if (data.current == undefined) {
      let snapshot = store.get_snapshot();
      data.current = snapshot;
    }
    return data.current!;
  }, [store, data, data.current]);

  return useSyncExternalStore(subscribe, getSnapshot, getSnapshot);
}

type Data = { [key: string]: unknown };
export type WbblNodeType = Node<Data>;
export function areNodePropsEqual(
  oldProps: NodeProps<WbblNodeType>,
  newProps: NodeProps<WbblNodeType>,
) {
  let shallowProps = [
    "id",
    "width",
    "height",
    "sourcePosition",
    "targetPosition",
    "selected",
    "dragHandle",
    "type",
    "dragging",
    "zIndex",
    "data",
  ];
  for (let prop in shallowProps) {
    if (oldProps[prop] !== newProps[prop]) {
      return false;
    }
  }
  let oldData = oldProps["data"];
  let newData = newProps["data"];
}
