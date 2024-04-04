import { ContextMenu } from "@radix-ui/themes";
import { PropsWithChildren, useMemo } from "react";

export default function GraphCanvasContextMenu(props: PropsWithChildren<{}>) {
  const contextMenuContent = useMemo(() => {
    return (
      <ContextMenu.Content>
        <ContextMenu.Item shortcut="⌘ E">Edit</ContextMenu.Item>
        <ContextMenu.Item shortcut="⌘ D">Duplicate</ContextMenu.Item>
        <ContextMenu.Separator />
        <ContextMenu.Item shortcut="⌘ N">Archive</ContextMenu.Item>
        <ContextMenu.Sub>
          <ContextMenu.SubTrigger>More</ContextMenu.SubTrigger>
          <ContextMenu.SubContent>
            <ContextMenu.Item>Move to project…</ContextMenu.Item>
            <ContextMenu.Item>Move to folder…</ContextMenu.Item>
            <ContextMenu.Separator />
            <ContextMenu.Item>Advanced options…</ContextMenu.Item>
          </ContextMenu.SubContent>
        </ContextMenu.Sub>
        <ContextMenu.Separator />
        <ContextMenu.Item>Share</ContextMenu.Item>
        <ContextMenu.Item>Add to favorites</ContextMenu.Item>
        <ContextMenu.Separator />
        <ContextMenu.Item shortcut="⌘ ⌫" color="red">
          Delete
        </ContextMenu.Item>
      </ContextMenu.Content>
    );
  }, []);
  return (
    <ContextMenu.Root>
      <ContextMenu.Trigger>{props.children}</ContextMenu.Trigger>
      {contextMenuContent}
    </ContextMenu.Root>
  );
}

GraphCanvasContextMenu;
