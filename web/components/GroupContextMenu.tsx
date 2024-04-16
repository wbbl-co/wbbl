import { ContextMenu } from "@radix-ui/themes";
import {
  MouseEventHandler,
  PropsWithChildren,
  memo,
  useCallback,
  useContext,
  useMemo,
} from "react";
import {
  WbblPreferencesStoreContext,
  useKeyBinding,
  useNodeKeyBinding,
} from "../hooks/use-preferences-store";
import { KeyboardShortcut, WbblWebappNodeType } from "../../pkg/wbbl";
import formatKeybinding from "../utils/format-keybinding";
import keybindingDescriptors from "../keybind-descriptors";
import MicroGroupIcon from "./icons/micro/MicroGroupIcon";
import MicroCleanIcon from "./icons/micro/MicroCleanIcon";
import MicroTrashIcon from "./icons/micro/MicroTrashIcon";
import MicroCutIcon from "./icons/micro/MicroCutIcon";
import MicroCopyPasteIcon from "./icons/micro/MicroCopyPasteIcon";
import MicroDuplicateIcon from "./icons/micro/MicroDuplicateIcon";
import { contextMenuContentPropDefs } from "@radix-ui/themes/props";
import { useElkJs } from "../hooks/use-elkjs";
import MicroUngroupIcon from "./icons/micro/MicroUngroupIcon";
import { useScopedShortcut } from "../hooks/use-shortcut";
import { WbblGraphStoreContext } from "../hooks/use-wbbl-graph-store";

function getSelectionCountLabel(edges: number, nodes: number) {
  if (nodes > 0 && edges > 0) {
    return `${nodes} node${nodes > 1 ? "s" : ""}, ${edges} edge${edges > 1 ? "s" : ""}`;
  } else if (edges > 0) {
    return `${edges} edge${edges > 1 ? "s" : ""}`;
  } else {
    return `${nodes} node${nodes > 1 ? "s" : ""}`;
  }
}

function GroupContextMenu(
  props: PropsWithChildren<{
    id: string;
    color: (typeof contextMenuContentPropDefs)["color"]["default"];
    nodes: string[];
    edges: string[];
  }>,
) {
  const preferencesStore = useContext(WbblPreferencesStoreContext);

  const makeJunctionShortcut = useNodeKeyBinding(
    preferencesStore,
    WbblWebappNodeType.Junction,
  );

  const helpShortcut = useKeyBinding(preferencesStore, KeyboardShortcut.Help);
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
  const ungroupShorcut = useKeyBinding(
    preferencesStore,
    KeyboardShortcut.UngroupNodes,
  );
  const blockNestedContextMenu = useCallback<MouseEventHandler>((evt) => {
    evt.preventDefault();
    evt.stopPropagation();
  }, []);

  const elkJs = useElkJs();
  const onElkJs = useCallback(() => {
    elkJs(new Set(props.nodes), new Set(props.edges));
  }, [elkJs, props.nodes, props.edges]);

  const graphStore = useContext(WbblGraphStoreContext);

  const deleteGroup = useCallback(() => {
    graphStore.remove_node_group_and_contents(props.id);
  }, [graphStore, props.id]);
  useScopedShortcut(KeyboardShortcut.Delete, deleteGroup, [
    graphStore,
    props.id,
  ]);

  const ungroup = useCallback(() => {
    graphStore.ungroup(props.id);
  }, [graphStore, props.id]);
  useScopedShortcut(KeyboardShortcut.UngroupNodes, ungroup, [
    graphStore,
    props.id,
  ]);

  const cut = useCallback(() => {
    graphStore.copy_group(props.id).then(() => {
      graphStore.remove_node_group_and_contents(props.id);
    });
  }, [graphStore, props.id]);
  useScopedShortcut(KeyboardShortcut.Cut, cut, [graphStore, props.id]);

  const copy = useCallback(() => {
    graphStore.copy_group(props.id);
  }, [graphStore, props.id]);
  useScopedShortcut(KeyboardShortcut.Copy, copy, [graphStore, props.id]);

  const duplicate = useCallback(() => {
    graphStore.duplicate_group(props.id);
  }, [graphStore, props.id]);
  useScopedShortcut(KeyboardShortcut.Duplicate, duplicate, [
    graphStore,
    props.id,
  ]);

  const contextMenuContent = useMemo(() => {
    return (
      <ContextMenu.Content
        onContextMenu={blockNestedContextMenu}
        onClick={blockNestedContextMenu}
        color={props.color}
        className={`group-context-menu`}
      >
        <ContextMenu.Item disabled>
          <MicroGroupIcon />
          {getSelectionCountLabel(props.edges.length, props.nodes.length)}
        </ContextMenu.Item>
        <ContextMenu.Separator />
        <ContextMenu.Item
          onClick={onElkJs}
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
        <ContextMenu.Item
          onClick={duplicate}
          shortcut={
            duplicateShortcut ? formatKeybinding(duplicateShortcut) : undefined
          }
        >
          <MicroDuplicateIcon /> Duplicate
        </ContextMenu.Item>
        <ContextMenu.Item
          shortcut={copyShortcut ? formatKeybinding(copyShortcut) : undefined}
        >
          <MicroCopyPasteIcon /> Copy
        </ContextMenu.Item>
        <ContextMenu.Item
          onClick={cut}
          shortcut={cutShortcut ? formatKeybinding(cutShortcut) : undefined}
        >
          <MicroCutIcon /> Cut
        </ContextMenu.Item>
        <ContextMenu.Separator />
        <ContextMenu.Item
          onClick={ungroup}
          shortcut={
            ungroupShorcut ? formatKeybinding(ungroupShorcut) : undefined
          }
        >
          <MicroUngroupIcon /> Ungroup
        </ContextMenu.Item>
        <ContextMenu.Item
          onClick={deleteGroup}
          shortcut={
            deleteShortcut ? formatKeybinding(deleteShortcut) : undefined
          }
          color="red"
        >
          <MicroTrashIcon /> Delete
        </ContextMenu.Item>
      </ContextMenu.Content>
    );
  }, [
    blockNestedContextMenu,
    cutShortcut,
    copyShortcut,
    helpShortcut,
    deleteShortcut,
    duplicateShortcut,
    makeJunctionShortcut,
    props.edges,
    props.nodes,
    onElkJs,
  ]);

  const menu = useMemo(
    () => (
      <ContextMenu.Root>
        <ContextMenu.Trigger>
          <g>{props.children}</g>
        </ContextMenu.Trigger>
        {contextMenuContent}
      </ContextMenu.Root>
    ),
    [props.children, contextMenuContent],
  );

  return menu;
}

export default memo(GroupContextMenu);
