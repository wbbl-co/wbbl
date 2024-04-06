import { LifebuoyIcon } from "@heroicons/react/24/solid";
import { ContextMenu } from "@radix-ui/themes";
import { PropsWithChildren, useContext, useMemo } from "react";
import { WbblGraphStoreContext } from "../hooks/use-wbbl-graph-store";

export default function GraphCanvasContextMenu(props: PropsWithChildren<{}>) {
  const graphStore = useContext(WbblGraphStoreContext);
  const contextMenuContent = useMemo(() => {
    return (
      <ContextMenu.Content>
        <ContextMenu.Item
          disabled={!graphStore.can_undo()}
          onClick={() => graphStore.undo()}
          shortcut="⌘ Z"
        >
          Undo
        </ContextMenu.Item>
        <ContextMenu.Item
          disabled={!graphStore.can_redo()}
          onClick={() => graphStore.redo()}
          shortcut="⇧⌘Z"
        >
          Redo
        </ContextMenu.Item>
        <ContextMenu.Separator />
        <ContextMenu.Item>
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
