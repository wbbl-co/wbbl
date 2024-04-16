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
import { useReactFlow } from "@xyflow/react";
import { JUNCTION_HEIGHT, JUNCTION_WIDTH } from "./node_types/JunctionNode";
import { isHotkeyPressed } from "react-hotkeys-hook";
import MicroGroupIcon from "./icons/micro/MicroGroupIcon";
import MicroCleanIcon from "./icons/micro/MicroCleanIcon";
import MicroTrashIcon from "./icons/micro/MicroTrashIcon";
import MicroHelpIcon from "./icons/micro/MicroHelpIcon";
import MicroAddIcon from "./icons/micro/MicroAddIcon";
import MicroUnbookmarkIcon from "./icons/micro/MicroUnbookmarkIcon";
import MicroBookmarkIcon from "./icons/micro/MicroBookmarkIcon";
import MicroCutIcon from "./icons/micro/MicroCutIcon";
import MicroCopyPasteIcon from "./icons/micro/MicroCopyPasteIcon";
import MicroDuplicateIcon from "./icons/micro/MicroDuplicateIcon";
import MicroPreviewIcon from "./icons/micro/MicroPreviewIcon";

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
      const selectedNodes = graphStore.get_locally_selected_nodes();
      const selectedEdges = graphStore.get_locally_selected_edges();
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
    {
      disabled:
        !currentNodeOrEdgeExclusivelySelected ||
        (!props.isEdge && !props.deleteable),
    },
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

  useScopedShortcut(
    KeyboardShortcut.LinkToPreview,
    () => {
      linkToPreview();
    },
    [graphStore, props.id, props.isEdge],
    {
      disabled:
        props.isEdge ||
        !currentNodeOrEdgeExclusivelySelected ||
        !props.previewable,
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
      const pos = flow.screenToFlowPosition(
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
              <MicroGroupIcon />
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
                  <MicroCleanIcon />
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
                <MicroPreviewIcon /> Link to Preview
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
              <MicroDuplicateIcon /> Duplicate
            </ContextMenu.Item>
            <ContextMenu.Item
              onClick={copyNodeOrSelection}
              shortcut={
                copyShortcut ? formatKeybinding(copyShortcut) : undefined
              }
            >
              <MicroCopyPasteIcon /> Copy
            </ContextMenu.Item>
            <ContextMenu.Item
              onClick={cutNodeOrSelection}
              shortcut={cutShortcut ? formatKeybinding(cutShortcut) : undefined}
            >
              <MicroCutIcon /> Cut
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
                {isFavouriteDeferred ? (
                  <MicroUnbookmarkIcon />
                ) : (
                  <MicroBookmarkIcon />
                )}
                {!isFavouriteDeferred ? "Bookmark" : "Unbookmark"}
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
              <MicroAddIcon /> Make Junction
            </ContextMenu.Item>
            <ContextMenu.Separator />
          </>
        ) : undefined}
        {currentNodeOrEdgeExclusivelySelected && (
          <ContextMenu.Item
            shortcut={helpShortcut ? formatKeybinding(helpShortcut) : undefined}
          >
            <MicroHelpIcon /> Help
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
              <MicroTrashIcon /> Delete
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
