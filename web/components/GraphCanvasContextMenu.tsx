import { LifebuoyIcon } from "@heroicons/react/24/solid";
import { ContextMenu } from "@radix-ui/themes";
import { PropsWithChildren, useContext, useMemo } from "react";
import { WbblGraphStoreContext } from "../hooks/use-wbbl-graph-store";
import {
  WbblPreferencesStoreContext,
  useKeyBinding,
} from "../hooks/use-preferences-store";
import { KeyboardShortcut } from "../../pkg/wbbl";
import formatKeybinding from "../utils/format-keybinding";

export default function GraphCanvasContextMenu(props: PropsWithChildren<{}>) {
  const graphStore = useContext(WbblGraphStoreContext);
  const preferencesStore = useContext(WbblPreferencesStoreContext);
  const undoBinding = useKeyBinding(preferencesStore, KeyboardShortcut.Undo);
  const redoBinding = useKeyBinding(preferencesStore, KeyboardShortcut.Redo);
  const helpBinding = useKeyBinding(preferencesStore, KeyboardShortcut.Help);
  const contextMenuContent = useMemo(() => {
    return (
      <ContextMenu.Content>
        <ContextMenu.Item
          disabled={!graphStore.can_undo()}
          onClick={() => graphStore.undo()}
          shortcut={undoBinding ? formatKeybinding(undoBinding) : undefined}
        >
          Undo
        </ContextMenu.Item>
        <ContextMenu.Item
          disabled={!graphStore.can_redo()}
          onClick={() => graphStore.redo()}
          shortcut={redoBinding ? formatKeybinding(redoBinding) : undefined}
        >
          Redo
        </ContextMenu.Item>
        <ContextMenu.Separator />
        <ContextMenu.Item
          shortcut={helpBinding ? formatKeybinding(helpBinding) : undefined}
        >
          <LifebuoyIcon width={"1em"} /> Help
        </ContextMenu.Item>
      </ContextMenu.Content>
    );
  }, [graphStore, graphStore.can_redo(), graphStore.can_undo()]);
  return (
    <ContextMenu.Root>
      <ContextMenu.Trigger>{props.children}</ContextMenu.Trigger>
      {contextMenuContent}
    </ContextMenu.Root>
  );
}

GraphCanvasContextMenu;
