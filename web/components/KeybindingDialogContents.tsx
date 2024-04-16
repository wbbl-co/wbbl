import {
  Kbd,
  Table,
  Heading,
  ScrollArea,
  Button,
  DropdownMenu,
  Flex,
  TextField,
  Dialog,
} from "@radix-ui/themes";
import {
  WbblPreferencesStoreContext,
  useKeyboardPreferences,
} from "../hooks/use-preferences-store";
import {
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  ChangeEvent,
} from "react";
import keybindingDescriptors from "../keybind-descriptors";
import { KeyboardShortcut } from "../../pkg/wbbl";
import { useRecordHotkeys } from "react-hotkeys-hook";
import formatKeybinding from "../utils/format-keybinding";
import normalizeKeybinding from "../utils/normalize-keybinding";
import { nodeMetaData, nodeTypes } from "./node_types";
import Fuse from "fuse.js";
import MicroTrashIcon from "./icons/micro/MicroTrashIcon";
import MicroSearchIcon from "./icons/micro/MicroSearchIcon";

export default function KeybindingDialogContents() {
  const store = useContext(WbblPreferencesStoreContext);
  const keyboardPreferences = useKeyboardPreferences(store);
  const [currentItem, setCurrentItem] = useState<
    | ["general_shortcut", KeyboardShortcut]
    | ["add_node", keyof typeof nodeTypes]
  >();

  type ShortcutMap =
    | {
        type: "general_shortcut";
        key: KeyboardShortcut;
        description: string;
        binding: string | undefined | null;
      }
    | {
        type: "add_node";
        key: keyof typeof nodeTypes;
        description: string;
        binding: string | undefined | null;
      };

  const keyboardBindingEntries = useMemo(() => {
    let shortcutMap: ShortcutMap[] = Object.entries(keybindingDescriptors).map(
      ([k, v]) => {
        const shortcut = KeyboardShortcut[
          Number(k) | 0
        ] as unknown as KeyboardShortcut;
        const binding = keyboardPreferences.keys.get(shortcut);

        return {
          type: "general_shortcut",
          key: Number(k) as KeyboardShortcut,
          description: v,
          binding,
        };
      },
    );
    const nodeShortcutMap: ShortcutMap[] = Object.entries(nodeMetaData).map(
      ([k, v]) => ({
        key: k as keyof typeof nodeTypes,
        type: "add_node",
        description: `Insert ${v.nodeMenuName ?? k} Node`,
        binding: keyboardPreferences.node_keys.get(k),
      }),
    );
    shortcutMap = shortcutMap.concat(nodeShortcutMap);
    return shortcutMap.sort((x, y) => {
      if (!x.binding && y.binding) {
        return 1;
      }
      if (!y.binding && x.binding) {
        return -1;
      }
      return x.description.localeCompare(y.description);
    });
  }, [keyboardPreferences]);

  const [searchTerm, setSearchTerm] = useState("");
  const index = useMemo(() => {
    return new Fuse(keyboardBindingEntries, {
      includeScore: true,
      keys: ["key", "description"],
    });
  }, [keyboardBindingEntries]);
  const searchResults = useMemo(() => {
    if (searchTerm.replace(/s+/, "") === "") {
      return undefined;
    }
    return index.search(searchTerm).map((x) => x.item);
  }, [index, searchTerm]);

  const [keys, { start, stop, isRecording }] = useRecordHotkeys();

  useEffect(() => {
    if (keys.has("enter")) {
      keys.delete("enter");
      if (keys.size > 0 && currentItem) {
        if (currentItem[0] === "general_shortcut") {
          store.set_keybinding(
            currentItem[1],
            normalizeKeybinding([...keys.values()].join("+")),
          );
        } else {
          store.set_node_keybinding(
            nodeMetaData[currentItem[1]].type,
            normalizeKeybinding([...keys.values()].join("+")),
          );
        }
      }
      stop();
    } else if (keys.has("escape")) {
      keys.delete("escape");
      stop();
    }
  }, [keys.size, currentItem]);

  const dropdownMenuContents = useMemo(() => {
    return (
      <DropdownMenu.Content>
        <DropdownMenu.Item onClick={() => start()}>
          Set Binding
        </DropdownMenu.Item>
        <DropdownMenu.Item
          onClick={
            currentItem !== undefined
              ? () => {
                  if (currentItem[0] == "general_shortcut") {
                    store.reset_keybinding(currentItem[1]);
                  } else {
                    store.reset_node_keybinding(
                      nodeMetaData[currentItem[1]].type,
                    );
                  }
                }
              : undefined
          }
        >
          Reset to Default
        </DropdownMenu.Item>
        <DropdownMenu.Item
          color="red"
          onClick={
            currentItem !== undefined
              ? () => {
                  if (currentItem[0] == "general_shortcut") {
                    store.set_keybinding(currentItem[1], undefined);
                  } else {
                    store.set_node_keybinding(
                      nodeMetaData[currentItem[1]].type,
                      undefined,
                    );
                  }
                }
              : undefined
          }
        >
          <MicroTrashIcon /> Remove Binding
        </DropdownMenu.Item>
      </DropdownMenu.Content>
    );
  }, [currentItem, store, keyboardPreferences, start]);

  const tableContents = useMemo(() => {
    return (searchResults ?? keyboardBindingEntries).map((entry) => (
      <DropdownMenu.Root
        key={entry.key}
        onOpenChange={(open) => {
          if (open) {
            if (entry.type === "add_node") {
              setCurrentItem([entry.type, entry.key]);
            } else {
              setCurrentItem([entry.type, entry.key]);
            }
          }
        }}
      >
        <Table.Row>
          <Table.RowHeaderCell style={{ textTransform: "capitalize" }}>
            {entry.description}
          </Table.RowHeaderCell>
          <Table.Cell>
            {currentItem && currentItem[1] === entry.key && isRecording ? (
              <Flex gap={"2"}>
                <Kbd style={{ minWidth: "4em" }} size={"3"}>
                  {keys.size === 0 ? (
                    <i>Waiting...</i>
                  ) : (
                    `${formatKeybinding([...keys.values()].join("+"))}${entry.type === "add_node" ? "+click" : ""}`
                  )}
                </Kbd>
                <Button
                  onClick={() => {
                    if (entry.type === "general_shortcut") {
                      store.set_keybinding(
                        entry.key,
                        normalizeKeybinding([...keys.values()].join("+")),
                      );
                    } else {
                      store.set_node_keybinding(
                        nodeMetaData[entry.key].type,
                        normalizeKeybinding([...keys.values()].join("+")),
                      );
                    }
                    stop();
                  }}
                  size={"1"}
                  color="lime"
                >
                  Save (enter)
                </Button>
                <Button onClick={stop} size={"1"} color="blue">
                  Cancel (esc)
                </Button>
              </Flex>
            ) : (
              <DropdownMenu.Trigger value={entry.key}>
                <Button asChild value={entry.key}>
                  <Kbd size={"3"}>
                    {!entry.binding
                      ? "Unset"
                      : `${formatKeybinding(entry.binding)}${entry.type === "add_node" ? "+click" : ""}`}
                  </Kbd>
                </Button>
              </DropdownMenu.Trigger>
            )}
          </Table.Cell>
        </Table.Row>
        {dropdownMenuContents}
      </DropdownMenu.Root>
    ));
  }, [
    keyboardBindingEntries,
    searchResults,
    setCurrentItem,
    dropdownMenuContents,
    isRecording,
    currentItem,
    keys,
    stop,
    store,
  ]);

  return (
    <Dialog.Content
      onEscapeKeyDown={useCallback(
        (evt: KeyboardEvent) => {
          if (isRecording) {
            evt.preventDefault();
          }
        },
        [isRecording],
      )}
      title={"Key Bindings"}
    >
      <Flex justify={"between"}>
        <Heading style={{ paddingBottom: "var(--space-3)" }} as="h2">
          Keyboard Shortcuts
        </Heading>
        <TextField.Root
          disabled={isRecording}
          placeholder="Search Shortcuts"
          autoFocus={true}
          onChange={useCallback(
            (evt: ChangeEvent<HTMLInputElement>) => {
              setSearchTerm(evt.target.value);
            },
            [setSearchTerm],
          )}
        >
          <TextField.Slot>
            <MicroSearchIcon />
          </TextField.Slot>
        </TextField.Root>
      </Flex>
      <ScrollArea
        type="hover"
        style={{
          paddingTop: "var(--space-3)",
          height: "min(400px, 75dvh)",
          overflowX: "hidden",
        }}
      >
        <Table.Root variant="ghost">
          <Table.Header>
            <Table.Row>
              <Table.ColumnHeaderCell width={"50%"}>
                Description
              </Table.ColumnHeaderCell>
              <Table.ColumnHeaderCell width={"50%"}>
                Binding (Click for options)
              </Table.ColumnHeaderCell>
            </Table.Row>
          </Table.Header>
          <Table.Body>{tableContents}</Table.Body>
        </Table.Root>
      </ScrollArea>
    </Dialog.Content>
  );
}
