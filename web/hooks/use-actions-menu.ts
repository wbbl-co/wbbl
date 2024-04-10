import { createContext } from "react";
import { KeyboardShortcut, WbblWebappNodeType } from "../../pkg/wbbl";

export type AvailableActions = {
  actions: Map<KeyboardShortcut, { f: () => void; scope: string }[]>;
  addNode?(
    nodeType: WbblWebappNodeType,
    screenX: number,
    screenY: number,
  ): void;
};

export const AvailableActionsContext = createContext<AvailableActions>({
  actions: new Map(),
});
