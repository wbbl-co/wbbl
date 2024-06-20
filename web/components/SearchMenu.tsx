import { UseComboboxProps, useCombobox } from "downshift";
import {
  memo,
  useCallback,
  useContext,
  useMemo,
  useRef,
  useState,
} from "react";
import { AvailableActionsContext } from "../hooks/use-actions-menu";
import Fuse from "fuse.js";
import keybindingDescriptors from "../keybind-descriptors";
import { KeyboardShortcut, WbblWebappNodeType } from "../../pkg/wbbl";
import { nodeMetaData } from "./node_types";
import { WbblPreferencesStoreContext } from "../hooks/use-preferences-store";
import { Dialog, Text, Flex, TextField, ScrollArea } from "@radix-ui/themes";
import formatKeybinding from "../utils/format-keybinding";
import { Callout } from "@radix-ui/themes";
import { useScopedShortcut } from "../hooks/use-shortcut";
import MicroWarniningIcon from "./icons/micro/MicroWarningIcon";
import MicroSearchIcon from "./icons/micro/MicroSearchIcon";

type ComboBoxItem =
  | {
      type: "shortcut";
      key: KeyboardShortcut;
      f: () => void;
      description: string;
      binding: string | null | undefined;
      tooltip?: string;
    }
  | {
      type: "add-node";
      key: WbblWebappNodeType;
      description: string;
      binding: string | null | undefined;
      tooltip: string;
    };

function ActionMenuCombobox(props: {
  close: () => void;
  mousePosition: { current: { x: number; y: number } };
}) {
  const preferencesStore = useContext(WbblPreferencesStoreContext);
  const availableActions = useContext(AvailableActionsContext);
  const data = useMemo<ComboBoxItem[]>(() => {
    let nodeItems: ComboBoxItem[] = [];
    if (availableActions.addNode) {
      const nodeBindings = preferencesStore!.get_node_keybindings() as Map<
        string,
        string | null | undefined
      >;
      nodeItems = Object.entries(nodeMetaData)
        .filter(
          ([k]) =>
            !nodeMetaData[k as keyof typeof nodeMetaData].hiddenFromNodeMenu,
        )
        .map(([k, v]) => ({
          type: "add-node" as const,
          key: v.type,
          description: `Insert ${nodeMetaData[k as keyof typeof nodeMetaData].nodeMenuName ?? k} Node`,
          tooltip: nodeMetaData[k as keyof typeof nodeMetaData].description,
          binding: nodeBindings.get(k),
        }));
    }
    const bindings = preferencesStore!.get_keybindings() as Map<
      KeyboardShortcut,
      string | null | undefined
    >;
    return [...availableActions.actions.entries()]
      .filter(([, values]) => {
        return values[values.length - 1] !== undefined;
      })
      .map<ComboBoxItem>(([key, values]) => ({
        type: "shortcut" as const,
        key,
        f: values[values.length - 1].f,
        description: keybindingDescriptors[key],
        binding: bindings.get(
          KeyboardShortcut[key] as unknown as KeyboardShortcut,
        ),
      }))
      .concat(nodeItems)
      .sort((a, b) => a.description.localeCompare(b.description));
  }, [availableActions, preferencesStore]);

  const index = useMemo(() => {
    return new Fuse(data, { keys: ["description", "tooltip"] });
  }, [data]);

  const [query, setQuery] = useState("");
  const items = useMemo<ComboBoxItem[]>(() => {
    if (query.length === 0) {
      return data;
    }
    return index.search(query).map((x) => ({ ...x.item }));
  }, [query]);
  const closing = useRef(false);

  const comboBoxProps: UseComboboxProps<ComboBoxItem> = useMemo(
    () => ({
      onInputValueChange({ inputValue }) {
        if (!closing.current) {
          setQuery(inputValue);
        }
      },
      onSelectedItemChange({ selectedItem }) {
        if (selectedItem.type === "shortcut") {
          selectedItem.f();
        } else if (availableActions.addNode) {
          availableActions.addNode(
            selectedItem.key,
            props.mousePosition.current.x,
            props.mousePosition.current.y,
          );
        }
        closing.current = true;
        props.close();
      },
      items,
      itemToString(item) {
        return item ? item.description : "";
      },
      isOpen: true,
    }),
    [
      data,
      index,
      items,
      props.close,
      availableActions,
      props.mousePosition,
      closing,
    ],
  );

  const { getMenuProps, getInputProps, highlightedIndex, getItemProps } =
    useCombobox(comboBoxProps);

  return (
    <>
      <TextField.Root
        size={"2"}
        className="action-menu-search"
        placeholder={"Search Actions"}
        {...getInputProps()}
        value={query}
      >
        <TextField.Slot>
          <MicroSearchIcon />
        </TextField.Slot>
      </TextField.Root>
      {items.length > 0 ? (
        <ScrollArea className="action-menu-list-container" {...getMenuProps()}>
          <ul className="action-menu-list">
            {items.map((item, index) => (
              <li
                data-highlighted={highlightedIndex === index}
                className={["action-menu-item rt-DropdownMenuItem"]
                  .filter((x) => !!x)
                  .join(" ")}
                key={`${item.type}-${item.key}`}
                {...getItemProps({ item, index })}
              >
                <Flex justify={"between"}>
                  <Text size={"2"} className="action-menu-title">
                    {item.description}
                  </Text>
                  {item.binding ? (
                    <Text className="action-menu-shortcut rt-BaseMenuShortcut">
                      {formatKeybinding(item.binding)}
                    </Text>
                  ) : undefined}
                </Flex>
              </li>
            ))}
          </ul>
        </ScrollArea>
      ) : (
        <Callout.Root
          className="action-menu-callout"
          color="lime"
          {...getMenuProps()}
        >
          <Callout.Icon>
            <MicroWarniningIcon />
          </Callout.Icon>
          <Callout.Text>No actions were found for this query</Callout.Text>
        </Callout.Root>
      )}
    </>
  );
}

function SearchMenu(props: {
  open: boolean;
  useMousePosition: boolean;
  mousePosition: { current: { x: number; y: number } };
  setActionMenuSettings: (settings: {
    open: boolean;
    useMousePosition: boolean;
  }) => void;
}) {
  const [position, setPosition] = useState<{
    left?: string | number;
    right?: string | number;
    top?: string | number;
    bottom?: string | number;
  }>({});

  const updatePosition = useCallback(() => {
    const result: typeof position = {};
    if (props.mousePosition.current.x > window.innerWidth - 400) {
      result.right = "1em";
    } else {
      result.left = props.mousePosition.current.x;
    }
    if (props.mousePosition.current.y > window.innerHeight - 600) {
      result.bottom = "1em";
    } else {
      result.top = props.mousePosition.current.y;
    }
    setPosition(result);
  }, [props.mousePosition, setPosition]);

  useScopedShortcut(
    KeyboardShortcut.QuickActions,
    () => {
      updatePosition();
      props.setActionMenuSettings({
        open: !props.open,
        useMousePosition: true,
      });
    },
    [props.setActionMenuSettings, props.open],
  );
  const onOpenChange = useCallback(
    (open: boolean) => {
      props.setActionMenuSettings({
        open: open,
        useMousePosition: props.useMousePosition,
      });
    },
    [props.mousePosition, props.useMousePosition, props.setActionMenuSettings],
  );

  const close = useCallback(() => {
    props.setActionMenuSettings({
      open: false,
      useMousePosition: props.useMousePosition,
    });
  }, [props.useMousePosition, props.setActionMenuSettings]);

  return (
    <Dialog.Root
      data-search-menu="true"
      open={props.open}
      onOpenChange={onOpenChange}
    >
      <Dialog.Content
        style={
          props.useMousePosition
            ? {
                position: "absolute",
                ...position,
              }
            : {}
        }
        className="action-menu"
      >
        <ActionMenuCombobox mousePosition={props.mousePosition} close={close} />
      </Dialog.Content>
    </Dialog.Root>
  );
}

export default memo(SearchMenu);
