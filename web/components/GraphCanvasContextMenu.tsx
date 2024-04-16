import { ContextMenu } from "@radix-ui/themes";
import {
  MouseEventHandler,
  PropsWithChildren,
  useCallback,
  useContext,
  useMemo,
} from "react";
import { WbblGraphStoreContext } from "../hooks/use-wbbl-graph-store";
import {
  WbblPreferencesStoreContext,
  useKeyBinding,
} from "../hooks/use-preferences-store";
import { KeyboardShortcut, WbblWebappGraphStore } from "../../pkg/wbbl";
import formatKeybinding from "../utils/format-keybinding";
import { useReactFlow } from "@xyflow/react";
import MicroUndoIcon from "./icons/micro/MicroUndoIcon";
import MicroRedoIcon from "./icons/micro/MicroRedoIcon";
import MicroCopyPasteIcon from "./icons/micro/MicroCopyPasteIcon";
import MicroHelpIcon from "./icons/micro/MicroHelpIcon";

export default function GraphCanvasContextMenu(
  props: PropsWithChildren<{
    mousePosition: { current: [number, number] };
  }>,
) {
  const flow = useReactFlow();
  const graphStore = useContext(WbblGraphStoreContext);
  const preferencesStore = useContext(WbblPreferencesStoreContext);
  const undoBinding = useKeyBinding(preferencesStore, KeyboardShortcut.Undo);
  const redoBinding = useKeyBinding(preferencesStore, KeyboardShortcut.Redo);
  const pasteBinding = useKeyBinding(preferencesStore, KeyboardShortcut.Paste);
  const helpBinding = useKeyBinding(preferencesStore, KeyboardShortcut.Help);
  const blockNestedContextMenu = useCallback<MouseEventHandler>((evt) => {
    evt.preventDefault();
  }, []);
  const contextMenuContent = useMemo(() => {
    return (
      <ContextMenu.Content onContextMenu={blockNestedContextMenu}>
        <ContextMenu.Item
          disabled={!graphStore.can_undo()}
          onClick={() => graphStore.undo()}
          shortcut={undoBinding ? formatKeybinding(undoBinding) : undefined}
        >
          <MicroUndoIcon />
          Undo
        </ContextMenu.Item>
        <ContextMenu.Item
          disabled={!graphStore.can_redo()}
          onClick={() => graphStore.redo()}
          shortcut={redoBinding ? formatKeybinding(redoBinding) : undefined}
        >
          <MicroRedoIcon />
          Redo
        </ContextMenu.Item>
        <ContextMenu.Separator />
        <ContextMenu.Item
          onClick={() => {
            const pos = flow.screenToFlowPosition({
              x: props.mousePosition.current[0],
              y: props.mousePosition.current[1],
            });
            WbblWebappGraphStore.get_clipboard_snapshot()
              .then((snapshot) =>
                graphStore.integrate_clipboard_snapshot(
                  snapshot,
                  new Float32Array([pos.x, pos.y]),
                ),
              )
              .catch(console.error);
          }}
          shortcut={pasteBinding ? formatKeybinding(pasteBinding) : undefined}
        >
          <MicroCopyPasteIcon />
          Paste
        </ContextMenu.Item>
        <ContextMenu.Separator />
        <ContextMenu.Item
          shortcut={helpBinding ? formatKeybinding(helpBinding) : undefined}
        >
          <MicroHelpIcon /> Help
        </ContextMenu.Item>
      </ContextMenu.Content>
    );
  }, [
    graphStore,
    graphStore.can_redo(),
    graphStore.can_undo(),
    blockNestedContextMenu,
    props.mousePosition,
    flow,
  ]);
  return (
    <ContextMenu.Root>
      <ContextMenu.Trigger>{props.children}</ContextMenu.Trigger>
      {contextMenuContent}
    </ContextMenu.Root>
  );
}

GraphCanvasContextMenu;
