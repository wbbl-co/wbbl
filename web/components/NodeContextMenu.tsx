import {
  ClipboardDocumentIcon,
  DocumentDuplicateIcon,
  EyeIcon,
  LifebuoyIcon,
  StarIcon,
  TrashIcon,
} from "@heroicons/react/24/solid";
import { ContextMenu } from "@radix-ui/themes";
import {
  PropsWithChildren,
  memo,
  useCallback,
  useContext,
  useMemo,
} from "react";
import { WbblGraphStoreContext } from "../hooks/use-wbbl-graph-store";
import { nodeMetaData } from "./node_types";

function NodeContextMenu(
  props: PropsWithChildren<{
    id: string;
    type: string;
    previewable: boolean;
    deleteable: boolean;
    copyable: boolean;
  }>,
) {
  const graphStore = useContext(WbblGraphStoreContext);
  const linkToPreview = useCallback(() => {
    graphStore.link_to_preview(props.id);
  }, [graphStore, props.id]);
  const deleteNode = useCallback(() => {
    graphStore.remove_node(props.id);
  }, [graphStore, props.id]);

  const contextMenuContent = useMemo(() => {
    return (
      <ContextMenu.Content
        className={`node-context-menu-content category-${nodeMetaData[props.type as keyof typeof nodeMetaData].category}`}
      >
        {props.previewable && (
          <>
            <ContextMenu.Item onClick={linkToPreview}>
              <EyeIcon width={"1em"} /> Link to Preview
            </ContextMenu.Item>
            <ContextMenu.Separator />
          </>
        )}
        {props.copyable && (
          <>
            <ContextMenu.Item shortcut="⌘ D">
              <DocumentDuplicateIcon width={"1em"} /> Duplicate
            </ContextMenu.Item>
            <ContextMenu.Item shortcut="⌘ C">
              <ClipboardDocumentIcon width={"1em"} /> Copy
            </ContextMenu.Item>
            <ContextMenu.Separator />
          </>
        )}
        {!nodeMetaData[props.type as keyof typeof nodeMetaData]
          .hiddenFromNodeMenu && (
          <>
            <ContextMenu.Item color="yellow">
              <StarIcon width={"1em"} /> Add to favorites
            </ContextMenu.Item>
          </>
        )}
        <ContextMenu.Item>
          <LifebuoyIcon width={"1em"} /> Help
        </ContextMenu.Item>
        {props.deleteable && (
          <>
            <ContextMenu.Separator />
            <ContextMenu.Item onClick={deleteNode} shortcut="⌘ ⌫" color="red">
              <TrashIcon width={"1em"} /> Delete
            </ContextMenu.Item>
          </>
        )}
      </ContextMenu.Content>
    );
  }, [
    deleteNode,
    linkToPreview,
    props.deleteable,
    props.copyable,
    props.previewable,
    props.type,
  ]);

  const nodeMenu = useMemo(
    () => (
      <ContextMenu.Root>
        <ContextMenu.Trigger>{props.children}</ContextMenu.Trigger>
        {contextMenuContent}
      </ContextMenu.Root>
    ),
    [props.children, contextMenuContent],
  );

  return nodeMenu;
}

export default memo(NodeContextMenu);
