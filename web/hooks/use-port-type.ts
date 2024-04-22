import { useNodeId } from "@xyflow/react";
import { useCallback, useMemo } from "react";
import {
  WbblWebappGraphSnapshot,
  useWbblGraphDataWithSelector,
} from "./use-wbbl-graph-store";

export default function usePortType(portId: `${"s" | "t"}#${number}`): unknown {
  const nodeId = useNodeId();
  const qualifiedId = useMemo(() => `${nodeId}#${portId}`, [nodeId, portId]);
  const getPortType = useCallback(
    (snapshot: WbblWebappGraphSnapshot) =>
      snapshot.computed_types?.get(qualifiedId),
    [qualifiedId],
  );
  return useWbblGraphDataWithSelector(getPortType);
}

export function usePortTypeWithNodeId(
  nodeId?: string,
  portId?: `${"s" | "t"}#${number}`,
): unknown {
  const qualifiedId = useMemo(() => `${nodeId}#${portId}`, [nodeId, portId]);
  const getPortType = useCallback(
    (snapshot: WbblWebappGraphSnapshot) => {
      return snapshot.computed_types?.get(qualifiedId);
    },
    [qualifiedId],
  );
  return useWbblGraphDataWithSelector(getPortType);
}
