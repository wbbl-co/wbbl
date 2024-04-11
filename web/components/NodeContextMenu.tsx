import {
  ClipboardDocumentIcon,
  DocumentDuplicateIcon,
  EyeIcon,
  LifebuoyIcon,
  StarIcon,
  TrashIcon,
  ScissorsIcon,
} from "@heroicons/react/24/solid";
import { ContextMenu } from "@radix-ui/themes";
import {
  MouseEventHandler,
  PropsWithChildren,
  memo,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
} from "react";
import { WbblGraphStoreContext } from "../hooks/use-wbbl-graph-store";
import { nodeMetaData } from "./node_types";
import { RectangleGroupIcon } from "@heroicons/react/16/solid";
import {
  WbblPreferencesStoreContext,
  useIsFavouritePreference,
  useKeyBinding,
} from "../hooks/use-preferences-store";
import { KeyboardShortcut } from "../../pkg/wbbl";
import formatKeybinding from "../utils/format-keybinding";
import { useScopedShortcut } from "../hooks/use-shortcut";
import { useElkJs } from "../hooks/use-elkjs";
import keybindingDescriptors from "../keybind-descriptors";
import { Squares2X2Icon } from "@heroicons/react/24/solid";

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
    selected: boolean;
  }>,
) {
  const graphStore = useContext(WbblGraphStoreContext);
  const preferencesStore = useContext(WbblPreferencesStoreContext);

  const linkToPreview = useCallback(() => {
    graphStore.link_to_preview(props.id);
  }, [graphStore, props.id]);
  const [isOpen, setIsOpen] = useState(false);

  const isFavourite = useIsFavouritePreference(
    preferencesStore,
    nodeMetaData[props.type as keyof typeof nodeMetaData].type,
    isOpen,
  );
  const [isFavouriteDeferred, setIsFavouriteDeferred] = useState(isFavourite);
  useEffect(() => {
    const handle = setTimeout(() => {
      setIsFavouriteDeferred(isFavourite);
    }, 100);
    return () => clearTimeout(handle);
  }, [isFavourite, setIsFavouriteDeferred]);
  const toggleFavourites = useCallback(() => {
    preferencesStore.set_favourite(
      nodeMetaData[props.type as keyof typeof nodeMetaData].type,
      !isFavourite,
    );
  }, [preferencesStore, props.type, isFavourite]);

  const [
    [selectedEdgesCount, selectedNodesCount, currentNodeExclusivelySelected],
    setCounts,
  ] = useState<[number, number, boolean]>([0, 0, false]);
  useEffect(() => {
    const handle = setTimeout(() => {
      let selectedNodes = graphStore.get_locally_selected_nodes();
      let edgesCount = graphStore.get_locally_selected_edges().length;
      setCounts([
        edgesCount,
        selectedNodes.length,
        (selectedNodes.length === 1 &&
          selectedNodes[0] === props.id &&
          edgesCount === 0) ||
          selectedNodes.length + edgesCount === 0 ||
          !props.selected,
      ]);
    }, 100);
    return () => clearTimeout(handle);
  }, [graphStore, isOpen, props.id, props.selected, setCounts]);

  const deleteNodeOrSelection = useCallback(
    currentNodeExclusivelySelected
      ? () => graphStore.remove_node(props.id)
      : () => graphStore.remove_selected_nodes_and_edges(),
    [props.id, graphStore, currentNodeExclusivelySelected],
  );

  const elkJs = useElkJs();

  useScopedShortcut(
    KeyboardShortcut.Delete,
    () => {
      graphStore.remove_node(props.id);
    },
    [graphStore, props.id],
    { disabled: !currentNodeExclusivelySelected },
  );

  const duplicateNodeOrSelection = useCallback(
    currentNodeExclusivelySelected
      ? () => graphStore.duplicate_node(props.id)
      : () => graphStore.duplicate(),
    [props.id, graphStore, currentNodeExclusivelySelected],
  );

  const copyNodeOrSelection = useCallback(
    currentNodeExclusivelySelected
      ? () => {
          graphStore.copy_node(props.id).catch(console.error);
        }
      : () => {
          graphStore.copy().catch(console.error);
        },
    [props.id, graphStore, currentNodeExclusivelySelected],
  );

  const cutNodeOrSelection = useCallback(
    currentNodeExclusivelySelected
      ? () => {
          graphStore.copy_node(props.id).then(() => {
            graphStore.remove_node(props.id);
          });
        }
      : () => {
          graphStore.copy().then(() => {
            graphStore.remove_selected_nodes_and_edges();
          });
        },
    [props.id, graphStore, currentNodeExclusivelySelected],
  );

  useScopedShortcut(
    KeyboardShortcut.Duplicate,
    () => {
      graphStore.duplicate_node(props.id);
    },
    [graphStore, props.id],
    { disabled: !currentNodeExclusivelySelected || !props.copyable },
  );

  useScopedShortcut(
    KeyboardShortcut.Copy,
    () => {
      graphStore.copy_node(props.id);
    },
    [graphStore, props.id],
    { disabled: !currentNodeExclusivelySelected || !props.copyable },
  );

  useScopedShortcut(
    KeyboardShortcut.Cut,
    () => {
      graphStore.copy_node(props.id).then(() => {
        graphStore.remove_node(props.id);
      });
    },
    [graphStore, props.id],
    { disabled: !currentNodeExclusivelySelected || !props.deleteable },
  );

  const helpShortcut = useKeyBinding(preferencesStore, KeyboardShortcut.Help);
  const linkToPreviewShortcut = useKeyBinding(
    preferencesStore,
    KeyboardShortcut.LinkToPreview,
  );
  const duplicateShortcut = useKeyBinding(
    preferencesStore,
    KeyboardShortcut.Duplicate,
  );
  const copyShortcut = useKeyBinding(preferencesStore, KeyboardShortcut.Copy);
  const cutShortcut = useKeyBinding(preferencesStore, KeyboardShortcut.Cut);
  const autoLayoutShortcut = useKeyBinding(
    preferencesStore,
    KeyboardShortcut.AutoLayout,
  );

  const deleteShortcut = useKeyBinding(
    preferencesStore,
    KeyboardShortcut.Delete,
  );
  const blockNestedContextMenu = useCallback<MouseEventHandler>((evt) => {
    evt.preventDefault();
    evt.stopPropagation();
  }, []);

  const contextMenuContent = useMemo(() => {
    return (
      <ContextMenu.Content
        onContextMenu={blockNestedContextMenu}
        onClick={blockNestedContextMenu}
        className={`node-context-menu-content category-${nodeMetaData[props.type as keyof typeof nodeMetaData].category}`}
      >
        {!currentNodeExclusivelySelected && (
          <>
            <ContextMenu.Item disabled>
              <RectangleGroupIcon width={"1em"} />
              {getSelectionCountLabel(selectedEdgesCount, selectedNodesCount)}
            </ContextMenu.Item>
            <ContextMenu.Separator />
            <ContextMenu.Item
              onClick={elkJs}
              shortcut={
                autoLayoutShortcut
                  ? formatKeybinding(autoLayoutShortcut)
                  : undefined
              }
            >
              <Squares2X2Icon width={"1em"} />
              {keybindingDescriptors[KeyboardShortcut.AutoLayout]}
            </ContextMenu.Item>
            <ContextMenu.Separator />
          </>
        )}
        {props.previewable && currentNodeExclusivelySelected && (
          <>
            <ContextMenu.Item
              shortcut={
                linkToPreviewShortcut
                  ? formatKeybinding(linkToPreviewShortcut)
                  : undefined
              }
              onClick={linkToPreview}
            >
              <EyeIcon width={"1em"} /> Link to Preview
            </ContextMenu.Item>
            <ContextMenu.Separator />
          </>
        )}
        {(!currentNodeExclusivelySelected || props.copyable) && (
          <>
            <ContextMenu.Item
              shortcut={
                duplicateShortcut
                  ? formatKeybinding(duplicateShortcut)
                  : undefined
              }
              onClick={duplicateNodeOrSelection}
            >
              <DocumentDuplicateIcon width={"1em"} /> Duplicate
            </ContextMenu.Item>
            <ContextMenu.Item
              onClick={copyNodeOrSelection}
              shortcut={
                copyShortcut ? formatKeybinding(copyShortcut) : undefined
              }
            >
              <ClipboardDocumentIcon width={"1em"} /> Copy
            </ContextMenu.Item>
            <ContextMenu.Item
              onClick={cutNodeOrSelection}
              shortcut={cutShortcut ? formatKeybinding(cutShortcut) : undefined}
            >
              <ScissorsIcon width={"1em"} /> Cut
            </ContextMenu.Item>
          </>
        )}
        {currentNodeExclusivelySelected &&
          !nodeMetaData[props.type as keyof typeof nodeMetaData]
            .hiddenFromNodeMenu && (
            <>
              <ContextMenu.Separator />
              <ContextMenu.Item onClick={toggleFavourites} color="yellow">
                <StarIcon width={"1em"} />
                {!isFavouriteDeferred ? "Favourite" : "Unfavourite"}
              </ContextMenu.Item>
            </>
          )}
        {currentNodeExclusivelySelected && (
          <ContextMenu.Item
            shortcut={helpShortcut ? formatKeybinding(helpShortcut) : undefined}
          >
            <LifebuoyIcon width={"1em"} /> Help
          </ContextMenu.Item>
        )}
        {(!currentNodeExclusivelySelected || props.deleteable) && (
          <>
            <ContextMenu.Separator />
            <ContextMenu.Item
              onClick={deleteNodeOrSelection}
              shortcut={
                deleteShortcut ? formatKeybinding(deleteShortcut) : undefined
              }
              color="red"
            >
              <TrashIcon width={"1em"} /> Delete
            </ContextMenu.Item>
          </>
        )}
      </ContextMenu.Content>
    );
  }, [
    selectedEdgesCount,
    deleteNodeOrSelection,
    selectedNodesCount,
    linkToPreview,
    toggleFavourites,
    props.deleteable,
    props.copyable,
    props.previewable,
    props.type,
    blockNestedContextMenu,
    isFavouriteDeferred,
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
