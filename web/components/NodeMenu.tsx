import {
  useCallback,
  useState,
  useMemo,
  KeyboardEvent as ReactKeyboardEvent,
  forwardRef,
  ForwardedRef,
  useEffect,
  useContext,
} from "react";
import { WbblWebappNodeType } from "../../pkg/wbbl";
import { NodeCategory, nodeMetaData } from "./node_types";
import { Text, DropdownMenu, Tooltip } from "@radix-ui/themes";
import {
  WbblPreferencesStoreContext,
  useFavouritesPreferences,
  useNodeKeyBinding,
} from "../hooks/use-preferences-store";
import formatKeybinding from "../utils/format-keybinding";
import MicroLandscapeIcon from "./icons/micro/MicroLandscapeIcon";
import MicroBookmarkIcon from "./icons/micro/MicroBookmarkIcon";

function useTooltipOpen() {
  const [tooltipMaybeOpen, setTooltipMaybeOpen] = useState(false);
  const [tooltipOpen, setTooltipOpen] = useState(false);

  const setTooltipMaybeOpenTrue = useCallback(() => {
    setTooltipMaybeOpen(true);
  }, [setTooltipMaybeOpen]);
  const setTooltipOpenFalse = useCallback(() => {
    setTooltipMaybeOpen(false);
    setTooltipOpen(false);
  }, [setTooltipMaybeOpen, setTooltipOpen]);
  useEffect(() => {
    if (tooltipMaybeOpen) {
      const handle = setTimeout(() => {
        setTooltipOpen(true);
      }, 250);
      return () => {
        clearTimeout(handle);
      };
    }
  }, [tooltipMaybeOpen, setTooltipOpen]);

  return [tooltipOpen, setTooltipMaybeOpenTrue, setTooltipOpenFalse] as const;
}

function NodeDropdownMenuItemImpl(
  {
    id,
    onSelect,
    value,
    color,
    onKeyEvent,
  }: {
    color?: boolean;
    id: string;
    onSelect: (key: string) => void;
    value: (typeof nodeMetaData)[keyof typeof nodeMetaData];
    onKeyEvent?: (evt: ReactKeyboardEvent<HTMLDivElement>) => void;
  },
  forwardRef: ForwardedRef<HTMLDivElement>,
) {
  const preferencesStore = useContext(WbblPreferencesStoreContext);
  const shortcut = useNodeKeyBinding(preferencesStore, value.type);
  const whenSelected = useCallback(() => {
    onSelect(id);
  }, [id, onSelect, value]);
  const [tooltipOpen, setTooltipMaybeOpenTrue, setTooltipOpenFalse] =
    useTooltipOpen();

  return (
    <div>
      <Tooltip open={tooltipOpen} content={value.description}>
        <DropdownMenu.Item
          shortcut={shortcut ? formatKeybinding(shortcut) : undefined}
          onBlur={setTooltipOpenFalse}
          onFocus={setTooltipMaybeOpenTrue}
          onMouseOver={setTooltipMaybeOpenTrue}
          onMouseLeave={setTooltipOpenFalse}
          ref={forwardRef}
          onKeyDown={onKeyEvent}
          textValue={id}
          onSelect={whenSelected}
          style={{
            textTransform: "capitalize",
            minWidth: 200,
            ...(color ? { color: `var(--${value.category}-color)` } : {}),
          }}
          key={id}
        >
          <Text>{value.nodeMenuName ?? id}</Text>
        </DropdownMenu.Item>
      </Tooltip>
    </div>
  );
}

function PreviewNodeDropdownMenuItemImpl(
  {
    onSelect,
    onKeyEvent,
  }: {
    onSelect: (key: string) => void;
    onKeyEvent?: (evt: ReactKeyboardEvent<HTMLDivElement>) => void;
  },
  forwardRef: ForwardedRef<HTMLDivElement>,
) {
  const whenSelected = useCallback(() => {
    onSelect("preview");
  }, [onSelect]);
  const [tooltipOpen, setTooltipMaybeOpenTrue, setTooltipOpenFalse] =
    useTooltipOpen();
  return (
    <div autoFocus={false}>
      <Tooltip
        open={tooltipOpen}
        content={nodeMetaData.preview.description}
        autoFocus={false}
      >
        <DropdownMenu.Item
          onBlur={setTooltipOpenFalse}
          onFocus={setTooltipMaybeOpenTrue}
          onMouseOver={setTooltipMaybeOpenTrue}
          onMouseLeave={setTooltipOpenFalse}
          ref={forwardRef}
          onKeyDown={onKeyEvent}
          className="category-utility preview-node-menu-item"
          onSelect={whenSelected}
        >
          <MicroLandscapeIcon />
          Preview
        </DropdownMenu.Item>
      </Tooltip>
    </div>
  );
}

const NodeDropdownMenuItem = forwardRef(NodeDropdownMenuItemImpl);
const PreviewNodeDropdownMenuItem = forwardRef(PreviewNodeDropdownMenuItemImpl);

export const NODE_MENU_DIMENSIONS = { width: 350, height: 400 } as const;
export default function NodeMenu(props: {
  open: boolean;
  onClose: (open: boolean) => void;
  position: null | {
    x: number;
    y: number;
    top?: number;
    left?: number;
    bottom?: number;
    right?: number;
  };
  addNode: (type: WbblWebappNodeType, x: number, y: number) => void;
}) {
  const sorted = useMemo(() => {
    return Object.entries(nodeMetaData)
      .filter(([, v]) => !v.hiddenFromNodeMenu)
      .sort(([k1, v1], [k2, v2]) =>
        (v1.nodeMenuName ?? k1).localeCompare(
          v2.nodeMenuName ?? k2,
          undefined,
          { usage: "search", collation: "phonebk" },
        ),
      );
  }, [nodeMetaData]);

  const grouped = useMemo(() => {
    const groups = sorted.reduce(
      (prev, curr) => {
        const category = curr[1].category;
        const categoryItems = prev[category] ?? [];
        prev[category] = categoryItems;
        categoryItems.push(
          curr as [
            keyof typeof nodeMetaData,
            (typeof nodeMetaData)[keyof typeof nodeMetaData],
          ],
        );
        return prev;
      },
      {} as {
        [K in NodeCategory]: [
          keyof typeof nodeMetaData,
          (typeof nodeMetaData)[keyof typeof nodeMetaData],
        ][];
      },
    );

    return Object.entries(groups).sort(([k1], [k2]) =>
      k1.localeCompare(k2, undefined, {
        usage: "search",
        collation: "phonebk",
      }),
    );
  }, [sorted]);

  const onSelect = useCallback(
    (evt: string) => {
      props.addNode(
        nodeMetaData[evt as keyof typeof nodeMetaData].type,
        props.position!.x,
        props.position!.y,
      );
      props.onClose(false);
    },
    [props.addNode, props.position, props.onClose, nodeMetaData],
  );
  const preferencesStore = useContext(WbblPreferencesStoreContext);
  const favourites = useFavouritesPreferences(preferencesStore);
  const mappedFavourites = useMemo(() => {
    return favourites.map((type) => {
      const value = Object.entries(nodeMetaData).filter(
        ([, v]) => v.type === type,
      )[0]!;
      return (
        <NodeDropdownMenuItem
          key={type}
          id={value[0]}
          value={value[1]}
          onSelect={onSelect}
        />
      );
    });
  }, [favourites, onSelect]);

  return (
    <DropdownMenu.Root open={props.open} onOpenChange={props.onClose}>
      <DropdownMenu.Content
        style={{
          ...props.position,
          position: "absolute",
          width: NODE_MENU_DIMENSIONS.width,
        }}
      >
        <PreviewNodeDropdownMenuItem onSelect={onSelect} />
        <DropdownMenu.Sub>
          <DropdownMenu.SubTrigger className="favourites-node-menu-item">
            <MicroBookmarkIcon />
            Bookmarks
          </DropdownMenu.SubTrigger>
          <DropdownMenu.SubContent>
            {favourites.length === 0 ? (
              <DropdownMenu.Item disabled={true}>
                No Bookmarks
              </DropdownMenu.Item>
            ) : (
              mappedFavourites
            )}
          </DropdownMenu.SubContent>
        </DropdownMenu.Sub>
        <DropdownMenu.Separator />
        {grouped.map(([key, values]) => (
          <DropdownMenu.Sub key={key}>
            <DropdownMenu.SubTrigger
              textValue={key.replace("-category", "")}
              style={{ textTransform: "capitalize" }}
            >
              <Text className={`node-menu__category-label category-${key}`}>
                ⬤
              </Text>
              {key.replace("-category", "")}
            </DropdownMenu.SubTrigger>
            <DropdownMenu.SubContent>
              {values.map(([key, value]) => (
                <NodeDropdownMenuItem
                  id={key}
                  key={key}
                  value={value}
                  onSelect={onSelect}
                />
              ))}
            </DropdownMenu.SubContent>
          </DropdownMenu.Sub>
        ))}
      </DropdownMenu.Content>
    </DropdownMenu.Root>
  );
}
