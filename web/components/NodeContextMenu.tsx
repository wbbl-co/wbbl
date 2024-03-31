
import { ClipboardDocumentIcon, DocumentDuplicateIcon, EyeIcon, LifebuoyIcon, StarIcon, TrashIcon } from "@heroicons/react/24/solid";
import { ContextMenu } from "@radix-ui/themes";
import { PropsWithChildren } from "react";


export function NodeContextMenu(props: PropsWithChildren<{}>) {
  return <ContextMenu.Root>
    <ContextMenu.Trigger>
      {props.children}
    </ContextMenu.Trigger>
    <ContextMenu.Content>
      <ContextMenu.Item><EyeIcon width={'1em'} /> Link to Preview</ContextMenu.Item>
      <ContextMenu.Separator />
      <ContextMenu.Item shortcut="⌘ D"><DocumentDuplicateIcon width={'1em'} /> Duplicate</ContextMenu.Item>
      <ContextMenu.Item shortcut="⌘ C"><ClipboardDocumentIcon width={'1em'} /> Copy</ContextMenu.Item>
      <ContextMenu.Separator />
      <ContextMenu.Item color="yellow" ><StarIcon width={'1em'} /> Add to favorites</ContextMenu.Item>
      <ContextMenu.Item><LifebuoyIcon width={'1em'} /> Help</ContextMenu.Item>
      <ContextMenu.Separator />
      <ContextMenu.Item shortcut="⌘ ⌫" color="red">
        <TrashIcon width={'1em'} /> Delete
      </ContextMenu.Item>
    </ContextMenu.Content>
  </ContextMenu.Root>
}
