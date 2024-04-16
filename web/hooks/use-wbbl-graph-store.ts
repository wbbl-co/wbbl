import { useCallback, useContext, useMemo, useRef } from "react";
import { useSyncExternalStore } from "react";
import { createContext } from "react";
import { WbblWebappGraphStore } from "../../pkg/wbbl";
import { Node, Edge } from "@xyflow/react";
import { graphWorker } from "../graph-worker-reference";

export const WbblGraphStoreContext = createContext<WbblWebappGraphStore>(
  WbblWebappGraphStore.empty(graphWorker),
);

export const WbblSnapshotContext = createContext<
  WbblWebappGraphSnapshot | undefined
>(undefined);

export type WbblWebappGraphSnapshot = {
  edges: Edge[];
  nodes: (Node & { groupId?: string })[];
  node_groups: {
    id: string;
    nodes: string[];
    path?: string;
    edges: string[];
    bounds: Float32Array;
  }[];
  computed_types: Map<string, unknown>;
};

export function useWbblGraphData(
  store: WbblWebappGraphStore,
): WbblWebappGraphSnapshot {
  const data = useRef<WbblWebappGraphSnapshot>();
  const count = useRef<number>(0);
  const cacheHandle = useRef<number>(0);
  const subscribe = useCallback(
    (subscriber: () => void) => {
      if (count.current == 0) {
        cacheHandle.current = store.subscribe(() => {
          data.current = undefined;
        });
      }
      count.current = count.current + 1;
      const handle = store.subscribe(subscriber);
      return () => {
        count.current = count.current - 1;
        if (count.current === 0) {
          store.unsubscribe(cacheHandle.current);
        }
        store.unsubscribe(handle);
      };
    },
    [store],
  );

  const getSnapshot = useCallback(() => {
    if (data.current == undefined) {
      const snapshot = store.get_snapshot();
      data.current = snapshot;
    }
    return data.current!;
  }, [store, data, data.current]);

  return useSyncExternalStore(subscribe, getSnapshot, getSnapshot);
}

export function useWbblGraphDataWithSelector<T>(
  selector: (snapshot: WbblWebappGraphSnapshot) => T,
): T | undefined {
  const snapshot = useContext(WbblSnapshotContext);
  return useMemo(() => {
    if (snapshot) {
      return selector(snapshot);
    }
    return undefined;
  }, [snapshot, selector]);
}

type Data = { [key: string]: unknown };
export type WbblNodeType = Node<Data>;

const shallowProps = [
  "id",
  "sourcePosition",
  "targetPosition",
  "positionAbsoluteX",
  "positionAbsoluteY",
  "selected",
  "dragHandle",
  "type",
  "dragging",
  "zIndex",
] as const;

export function areNodePropsEqual(
  oldProps: { [K in (typeof shallowProps)[any]]?: any } & {
    data: any;
  },
  newProps: { [K in (typeof shallowProps)[any]]?: any } & {
    data: any;
  },
) {
  for (const prop of shallowProps) {
    if (oldProps[prop] !== newProps[prop]) {
      return false;
    }
  }

  if (oldProps.data.size !== newProps.data.size) {
    return false;
  }
  for (const key of oldProps.data.keys()) {
    const oldValue = oldProps.data.get(key);
    const newValue = newProps.data.get(key);
    if (typeof oldValue == "object") {
      for (const subkey of Object.keys(oldValue as object)) {
        const oldSubValue = (oldValue as Record<string, unknown>)[subkey];
        const newSubValue = (newValue as Record<string, unknown>)[subkey];
        if (oldSubValue !== newSubValue) {
          return false;
        }
      }
      return false;
    } else if (oldValue !== newValue) {
      return false;
    }
  }

  return true;
}
