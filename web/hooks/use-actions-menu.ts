import { createContext } from "react";
import { KeyboardShortcut } from "../../pkg/wbbl";

export type AvailableActions = {
  actions: Map<KeyboardShortcut, { f: () => void; scope: string }[]>;
};
export const AvailableActionsContext = createContext<AvailableActions>({
  actions: new Map(),
});
