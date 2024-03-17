import { useContext } from "react";
import { useSyncExternalStore } from "react";
import { useSyncExternalStoreWithSelector } from "use-sync-external-store/with-selector";
import { createContext } from "react";
import {
  WbblWebappGraphSnapshot,
  WbblWebappGraphStore,
  WbblWebappData,
} from "../pkg/wbbl";
import { NodeProps, Node } from "@xyflow/react";

export const WbblGraphStoreContext = createContext<WbblWebappGraphStore>(
  WbblWebappGraphStore.empty(),
);

export function useWbblGraphDataWithSelector<T>(
  selector: (snapshot: WbblWebappGraphSnapshot) => T,
  isEqual: (a: T, b: T) => boolean,
): T {
  let store = useContext(WbblGraphStoreContext);
  let result = useSyncExternalStoreWithSelector(
    (subscriber) => {
      let handle = store.subscribe(subscriber);
      return () => store.unsubscribe(handle);
    },
    store.get_snapshot,
    store.get_snapshot,
    selector,
    isEqual,
  );

  return result;
}

export function useWbblGraphData(): WbblWebappGraphSnapshot {
  let store = useContext(WbblGraphStoreContext);
  let result = useSyncExternalStore(
    (subscriber) => {
      let handle = store.subscribe(subscriber);
      return () => store.unsubscribe(handle);
    },
    store.get_snapshot,
    store.get_snapshot,
  );

  return result;
}

type Data = WbblWebappData & { [key: string]: unknown };
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
  ];
  for (let prop in shallowProps) {
    if (oldProps[prop] !== newProps[prop]) {
      return false;
    }
  }
  let oldData = oldProps["data"];
  let newData = newProps["data"];
  return WbblWebappData.eq(oldData, newData);
}
