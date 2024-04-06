import {
  ClipboardDocumentIcon,
  DocumentDuplicateIcon,
  EyeIcon,
  LifebuoyIcon,
  StarIcon,
  TrashIcon,
} from "@heroicons/react/24/solid";
import { ContextMenu, Text } from "@radix-ui/themes";
import {
  PropsWithChildren,
  memo,
  useCallback,
  useContext,
  useMemo,
  useState,
} from "react";
import { WbblGraphStoreContext } from "../hooks/use-wbbl-graph-store";
import { nodeMetaData } from "./node_types";
import { RectangleGroupIcon } from "@heroicons/react/16/solid";

function getSelectionCountLabel(edges: number, nodes: number) {
  if (nodes > 0 && edges > 0) {
    return `${nodes} node${nodes > 1 ? "s" : ""}, ${edges} edge${edges > 1 ? "s" : ""}`;
  } else if (edges > 0) {
    return `${edges} edge${edges > 1 ? "s" : ""}`;
  } else {
    return `${nodes} node${nodes > 1 ? "s" : ""}`;
  }
}

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
  const [isOpen, setIsOpen] = useState(false);
  const [
    selectedEdgesCount,
    selectedNodesCount,
    currentNodeExclusivelySelected,
  ] = useMemo(() => {
    let selectedNodes = graphStore.get_locally_selected_nodes();
    let edgesCount = graphStore.get_locally_selected_edges().length;
    return [
      edgesCount,
      selectedNodes.length,
      (selectedNodes.length === 1 &&
        selectedNodes[0] === props.id &&
        edgesCount === 0) ||
        selectedNodes.length + edgesCount === 0,
    ];
  }, [isOpen, props.id]);

  const contextMenuContent = useMemo(() => {
    return (
      <ContextMenu.Content
        className={`node-context-menu-content category-${nodeMetaData[props.type as keyof typeof nodeMetaData].category}`}
      >
        {!currentNodeExclusivelySelected && (
          <>
            <ContextMenu.Item disabled>
              <RectangleGroupIcon width={"1em"} />
              {getSelectionCountLabel(selectedEdgesCount, selectedNodesCount)}
            </ContextMenu.Item>
          </>
        )}
        {props.previewable && currentNodeExclusivelySelected && (
          <>
            <ContextMenu.Item onClick={linkToPreview}>
              <EyeIcon width={"1em"} /> Link to Preview
            </ContextMenu.Item>
          </>
        )}
        {(!currentNodeExclusivelySelected || props.copyable) && (
          <>
            <ContextMenu.Separator />
            <ContextMenu.Item shortcut="⌘ D">
              <DocumentDuplicateIcon width={"1em"} /> Duplicate
            </ContextMenu.Item>
            <ContextMenu.Item shortcut="⌘ C">
              <ClipboardDocumentIcon width={"1em"} /> Copy
            </ContextMenu.Item>
          </>
        )}
        {currentNodeExclusivelySelected &&
          !nodeMetaData[props.type as keyof typeof nodeMetaData]
            .hiddenFromNodeMenu && (
            <>
              <ContextMenu.Separator />
              <ContextMenu.Item color="yellow">
                <StarIcon width={"1em"} /> Add to favorites
              </ContextMenu.Item>
            </>
          )}
        {currentNodeExclusivelySelected && (
          <ContextMenu.Item>
            <LifebuoyIcon width={"1em"} /> Help
          </ContextMenu.Item>
        )}
        {(!currentNodeExclusivelySelected || props.deleteable) && (
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
    selectedEdgesCount,
    deleteNode,
    selectedNodesCount,
    linkToPreview,
    props.deleteable,
    props.copyable,
    props.previewable,
    props.type,
  ]);

  const nodeMenu = useMemo(
    () => (
      <ContextMenu.Root onOpenChange={setIsOpen}>
        <ContextMenu.Trigger>{props.children}</ContextMenu.Trigger>
        {contextMenuContent}
      </ContextMenu.Root>
    ),
    [props.children, contextMenuContent, setIsOpen],
  );

  return nodeMenu;
}

export default memo(NodeContextMenu);
