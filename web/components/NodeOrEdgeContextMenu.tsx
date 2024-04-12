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
  useNodeKeyBinding,
} from "../hooks/use-preferences-store";
import { KeyboardShortcut, WbblWebappNodeType } from "../../pkg/wbbl";
import formatKeybinding from "../utils/format-keybinding";
import { useScopedShortcut } from "../hooks/use-shortcut";
import { useElkJs } from "../hooks/use-elkjs";
import keybindingDescriptors from "../keybind-descriptors";
import { Squares2X2Icon } from "@heroicons/react/24/solid";
import { useReactFlow } from "@xyflow/react";
import { PlusIcon } from "@heroicons/react/24/solid";
import { JUNCTION_HEIGHT, JUNCTION_WIDTH } from "./node_types/JunctionNode";
import { isHotkeyPressed } from "react-hotkeys-hook";

function getSelectionCountLabel(edges: number, nodes: number) {
  if (nodes > 0 && edges > 0) {
    return `${nodes} node${nodes > 1 ? "s" : ""}, ${edges} edge${edges > 1 ? "s" : ""}`;
  } else if (edges > 0) {
    return `${edges} edge${edges > 1 ? "s" : ""}`;
  } else {
    return `${nodes} node${nodes > 1 ? "s" : ""}`;
  }
}

function NodeOrEdgeContextMenu(
  props: PropsWithChildren<
    { id: string; selected: boolean } & (
      | { isEdge: true; edgeClassname: string }
      | {
          isEdge: false;
          type: string;
          previewable: boolean;
          deleteable: boolean;
          copyable: boolean;
        }
    )
  >,
) {
  const graphStore = useContext(WbblGraphStoreContext);
  const preferencesStore = useContext(WbblPreferencesStoreContext);

  const linkToPreview = useCallback(() => {
    graphStore.link_to_preview(props.id);
  }, [graphStore, props.id]);
  const [isOpen, setIsOpen] = useState(false);

  const isFavourite = useIsFavouritePreference(
    preferencesStore,
    props.isEdge
      ? undefined
      : nodeMetaData[props.type as keyof typeof nodeMetaData].type,
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
    if (!props.isEdge) {
      preferencesStore.set_favourite(
        nodeMetaData[props.type as keyof typeof nodeMetaData].type,
        !isFavourite,
      );
    }
  }, [preferencesStore, !props.isEdge && props.type, isFavourite]);

  const [
    [
      selectedEdgesCount,
      selectedNodesCount,
      currentNodeOrEdgeExclusivelySelected,
    ],
    setCounts,
  ] = useState<[number, number, boolean]>([0, 0, false]);
  useEffect(() => {
    const handle = setTimeout(() => {
      let selectedNodes = graphStore.get_locally_selected_nodes();
      let selectedEdges = graphStore.get_locally_selected_edges();
      if (props.isEdge) {
        setCounts([
          selectedEdges.length,
          selectedNodes.length,
          (selectedEdges.length === 1 &&
            selectedEdges[0] === props.id &&
            selectedNodes.length === 0) ||
            selectedNodes.length + selectedEdges.length === 0 ||
            !props.selected,
        ]);
      } else {
        setCounts([
          selectedEdges.length,
          selectedNodes.length,
          (selectedNodes.length === 1 &&
            selectedNodes[0] === props.id &&
            selectedEdges.length === 0) ||
            selectedNodes.length + selectedEdges.length === 0 ||
            !props.selected,
        ]);
      }
    }, 100);
    return () => clearTimeout(handle);
  }, [graphStore, isOpen, props.id, props.selected, setCounts, props.isEdge]);

  const deleteNodeOrSelection = useCallback(
    currentNodeOrEdgeExclusivelySelected
      ? () => {
          if (props.isEdge) {
            graphStore.remove_edge(props.id);
          } else {
            graphStore.remove_node(props.id);
          }
        }
      : () => graphStore.remove_selected_nodes_and_edges(),
    [props.id, graphStore, currentNodeOrEdgeExclusivelySelected, props.isEdge],
  );

  const elkJs = useElkJs();

  useScopedShortcut(
    KeyboardShortcut.Delete,
    () => {
      if (props.isEdge) {
        graphStore.remove_edge(props.id);
      } else {
        graphStore.remove_node(props.id);
      }
    },
    [graphStore, props.id, props.isEdge],
    { disabled: !currentNodeOrEdgeExclusivelySelected },
  );

  const duplicateNodeOrSelection = useCallback(
    currentNodeOrEdgeExclusivelySelected
      ? () => graphStore.duplicate_node(props.id)
      : () => graphStore.duplicate(),
    [props.id, graphStore, currentNodeOrEdgeExclusivelySelected],
  );

  const copyNodeOrSelection = useCallback(
    currentNodeOrEdgeExclusivelySelected
      ? () => {
          graphStore.copy_node(props.id).catch(console.error);
        }
      : () => {
          graphStore.copy().catch(console.error);
        },
    [props.id, graphStore, currentNodeOrEdgeExclusivelySelected],
  );

  const cutNodeOrSelection = useCallback(
    currentNodeOrEdgeExclusivelySelected
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
    [props.id, graphStore, currentNodeOrEdgeExclusivelySelected],
  );

  useScopedShortcut(
    KeyboardShortcut.Duplicate,
    () => {
      graphStore.duplicate_node(props.id);
    },
    [graphStore, props.id],
    {
      disabled:
        !currentNodeOrEdgeExclusivelySelected ||
        (!props.isEdge && !props.copyable),
    },
  );

  useScopedShortcut(
    KeyboardShortcut.Copy,
    () => {
      graphStore.copy_node(props.id);
    },
    [graphStore, props.id],
    {
      disabled:
        !currentNodeOrEdgeExclusivelySelected ||
        (!props.isEdge && !props.copyable),
    },
  );

  useScopedShortcut(
    KeyboardShortcut.Cut,
    () => {
      if (!props.isEdge) {
        graphStore.copy_node(props.id).then(() => {
          graphStore.remove_node(props.id);
        });
      } else {
        graphStore.remove_edge(props.id);
      }
    },
    [graphStore, props.id, props.isEdge],
    {
      disabled:
        !currentNodeOrEdgeExclusivelySelected ||
        (!props.isEdge && !props.deleteable),
    },
  );

  const makeJunctionShortcut = useNodeKeyBinding(
    preferencesStore,
    WbblWebappNodeType.Junction,
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

  const flow = useReactFlow();
  const makeJunction = useCallback<MouseEventHandler>(
    (evt) => {
      let pos = flow.screenToFlowPosition(
        {
          x: evt.clientX - JUNCTION_WIDTH / 2,
          y: evt.clientY - JUNCTION_HEIGHT / 2,
        },
        { snapToGrid: false },
      );
      graphStore.make_junction(props.id, pos.x, pos.y);
    },
    [flow, graphStore, props.id],
  );

  const contextMenuContent = useMemo(() => {
    return (
      <ContextMenu.Content
        onContextMenu={blockNestedContextMenu}
        onClick={blockNestedContextMenu}
        className={` ${props.isEdge ? `edge-context-menu-content ${props.edgeClassname}` : `node-context-menu-content category-${nodeMetaData[props.type as keyof typeof nodeMetaData].category}`}`}
      >
        {!currentNodeOrEdgeExclusivelySelected && (
          <>
            <ContextMenu.Item disabled>
              <RectangleGroupIcon width={"1em"} />
              {getSelectionCountLabel(selectedEdgesCount, selectedNodesCount)}
            </ContextMenu.Item>
            <ContextMenu.Separator />
            {selectedNodesCount > 1 ? (
              <>
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
            ) : undefined}
          </>
        )}
        {!props.isEdge &&
          props.previewable &&
          currentNodeOrEdgeExclusivelySelected && (
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
        {(!currentNodeOrEdgeExclusivelySelected ||
          (!props.isEdge && props.copyable)) && (
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
        {currentNodeOrEdgeExclusivelySelected &&
          !props.isEdge &&
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
        {currentNodeOrEdgeExclusivelySelected && props.isEdge ? (
          <>
            <ContextMenu.Item
              onClick={makeJunction}
              shortcut={
                makeJunctionShortcut
                  ? `${formatKeybinding(makeJunctionShortcut)}+click`
                  : undefined
              }
            >
              <PlusIcon width={"1em"} /> Make Junction
            </ContextMenu.Item>
            <ContextMenu.Separator />
          </>
        ) : undefined}
        {currentNodeOrEdgeExclusivelySelected && (
          <ContextMenu.Item
            shortcut={helpShortcut ? formatKeybinding(helpShortcut) : undefined}
          >
            <LifebuoyIcon width={"1em"} /> Help
          </ContextMenu.Item>
        )}

        {(!currentNodeOrEdgeExclusivelySelected ||
          props.isEdge ||
          props.deleteable) && (
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
    props.isEdge || props.deleteable,
    props.isEdge || props.copyable,
    props.isEdge || props.previewable,
    props.isEdge || props.type,
    blockNestedContextMenu,
    isFavouriteDeferred,
    cutShortcut,
    copyShortcut,
    helpShortcut,
    deleteShortcut,
    duplicateShortcut,
    makeJunctionShortcut,
  ]);

  const onClick = useCallback<MouseEventHandler>(
    (evt) => {
      if (makeJunctionShortcut && isHotkeyPressed(makeJunctionShortcut)) {
        makeJunction(evt);
        evt.preventDefault();
        evt.stopPropagation();
      }
    },
    [makeJunction, makeJunctionShortcut],
  );

  const menu = useMemo(
    () => (
      <ContextMenu.Root onOpenChange={setIsOpen}>
        <ContextMenu.Trigger>
          {props.isEdge ? (
            <g onClick={onClick}>{props.children}</g>
          ) : (
            props.children
          )}
        </ContextMenu.Trigger>
        {contextMenuContent}
      </ContextMenu.Root>
    ),
    [props.children, contextMenuContent, setIsOpen],
  );

  return menu;
}

export default memo(NodeOrEdgeContextMenu);
