import { ChangeEvent, useCallback, useState, useMemo, useRef, KeyboardEvent as ReactKeyboardEvent } from "react";
import { WbblWebappNodeType } from "../../pkg/wbbl";
import { NodeCategory, nodeMetaData } from "./node_types";
import { TextField, Text, DropdownMenu, Box, Callout } from "@radix-ui/themes";
import { MagnifyingGlassIcon, StarIcon, PhotoIcon, InformationCircleIcon } from "@heroicons/react/24/solid";
import { ScrollArea } from "@radix-ui/themes/dist/cjs/index.js";



function NodeDropdownMenuItem({ id, onSelect, value, color, onKeyEvent }: { color?: boolean, id: string, onSelect: (key: string) => void, value: (typeof nodeMetaData)[keyof typeof nodeMetaData], onKeyEvent?: (evt: ReactKeyboardEvent<HTMLDivElement>) => void }) {
  const whenSelected = useCallback(() => { onSelect(id) }, [id, onSelect, value]);
  return <DropdownMenu.Item onKeyDown={onKeyEvent} textValue={id} onSelect={whenSelected} style={{ textTransform: 'capitalize', minWidth: 200, ...(color ? { color: `var(--${value.category}-color)` } : {}) }} key={id}>{value.nodeMenuName ?? id}</DropdownMenu.Item>;
}

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
    return Object.entries(nodeMetaData).filter(([, v]) => !v.hiddenFromNodeMenu).sort(([k1, v1], [k2, v2]) => (v1.nodeMenuName ?? k1).localeCompare((v2.nodeMenuName ?? k2), undefined, { usage: 'search', collation: 'phonebk' }))
  }, [nodeMetaData]);



  const grouped = useMemo(() => {
    let groups = sorted.reduce((prev, curr) => {
      let category = curr[1].category;
      let categoryItems = prev[category] ?? [];
      prev[category] = categoryItems;
      categoryItems.push(curr as [keyof typeof nodeMetaData, (typeof nodeMetaData)[keyof typeof nodeMetaData]]);
      return prev;
    }, {} as { [K in NodeCategory]: [keyof typeof nodeMetaData, (typeof nodeMetaData)[keyof typeof nodeMetaData]][] });

    return Object.entries(groups).sort(([k1,], [k2,]) => k1.localeCompare(k2, undefined, { usage: 'search', collation: 'phonebk' }))
  }, [sorted]);

  const [query, setQuery] = useState("");

  const onClose = useCallback((b: boolean) => {
    if (!b) {
      setQuery("");
    }
    props.onClose(b);
  }, [props.onClose]);

  const filteredItems = useMemo(() =>
    query === ""
      ? null
      : sorted.filter(([key, item]) => {
        return (
          (item.nodeMenuName ?? key).toLowerCase().includes(query.toLowerCase()) ||
          item.description.toLowerCase().includes(query.toLowerCase())
        );
      }),
    [query, sorted]);

  const onSelect = useCallback(
    (evt: string) => {
      props.addNode(nodeMetaData[evt as keyof typeof nodeMetaData].type, props.position!.x, props.position!.y);
      props.onClose(false);
      setQuery("");
    },
    [props.addNode, props.position, props.onClose, setQuery, nodeMetaData],
  );



  const onSelectFirst = useCallback(
    () => {
      if (filteredItems == null) {
        let element = (grouped[0][1][0])[1];
        props.addNode(element.type, props.position!.x, props.position!.y);
        props.onClose(false);
        setQuery("");
      } else {
        let items = (filteredItems == null ? sorted : filteredItems);
        if (items.length > 0) {
          props.addNode(items[0][1].type, props.position!.x, props.position!.y);
          props.onClose(false);
          setQuery("");
        }
      }
    },
    [props.addNode, props.position, props.onClose, filteredItems, grouped, setQuery],
  );


  const onSelectPreview = useCallback(
    () => {
      props.addNode(WbblWebappNodeType.Preview, props.position!.x, props.position!.y);
      props.onClose(false);
      setQuery("");
    },
    [props.addNode, props.position, props.onClose, setQuery],
  );

  const updateQuery = useCallback((evt: ChangeEvent<HTMLInputElement>) => {
    setQuery(evt.target.value)
  }, [setQuery]);

  const dropdownMenuRef = useRef<HTMLDivElement | null>(null);
  const searchBoxRef = useRef<HTMLInputElement | null>(null);

  const keydownFromSearch = useCallback((evt: ReactKeyboardEvent<HTMLDivElement>) => {
    if (evt.key === "ArrowDown" && dropdownMenuRef.current && (filteredItems == null || filteredItems.length > 0)) {
      dropdownMenuRef.current.focus();
      evt.preventDefault();
    }
  }, [dropdownMenuRef.current, filteredItems]);

  const keydownUpFromMenu = useCallback((evt: ReactKeyboardEvent<HTMLDivElement>) => {
    if (evt.key === "ArrowUp" && searchBoxRef.current) {
      searchBoxRef.current.focus();
      evt.preventDefault();
    }
  }, [searchBoxRef.current]);


  return (
    <DropdownMenu.Root open={props.open} onOpenChange={onClose}>
      <DropdownMenu.Content ref={dropdownMenuRef} style={{ ...props.position, position: 'absolute', width: NODE_MENU_DIMENSIONS.width }}>
        <Box p='3'>
          <TextField.Root ref={searchBoxRef} autoFocus onKeyDown={keydownFromSearch} size='3' color="red" onSubmit={onSelectFirst} onChange={updateQuery} placeholder="Search for nodes…">
            <TextField.Slot>
              <MagnifyingGlassIcon height="16" width="16" />
            </TextField.Slot>
          </TextField.Root>
        </Box>
        <ScrollArea type="hover" scrollbars="vertical" style={{ maxHeight: NODE_MENU_DIMENSIONS.height }}>
          {filteredItems == null ?
            (<>
              <DropdownMenu.Item onKeyDown={keydownUpFromMenu} onSelect={onSelectPreview}><PhotoIcon color="current" width={'1em'} height={'1em'} /> Add Preview</DropdownMenu.Item>
              <DropdownMenu.Sub>
                <DropdownMenu.SubTrigger><StarIcon color="current" width={'1em'} height={'1em'} /> Favourites</DropdownMenu.SubTrigger>
                <DropdownMenu.SubContent>
                  <DropdownMenu.Item color="gray">No Favourites</DropdownMenu.Item>
                </DropdownMenu.SubContent>
              </DropdownMenu.Sub>
              <DropdownMenu.Separator />
              {grouped.map(([key, values]) => (
                <DropdownMenu.Sub key={key}>
                  <DropdownMenu.SubTrigger style={{ textTransform: 'capitalize' }}><Text style={{ color: `var(--${key}-color)` }}>⬤</Text>{key}</DropdownMenu.SubTrigger>
                  <DropdownMenu.SubContent>
                    {values.map(([key, value]) => <NodeDropdownMenuItem id={key} key={key} value={value} onSelect={onSelect} />)}
                  </DropdownMenu.SubContent>
                </DropdownMenu.Sub>
              )
              )}
            </>)

            : (filteredItems.length > 0
              ? <>{filteredItems.map(([key, value], idx) => <NodeDropdownMenuItem id={key} key={key} value={value} onSelect={onSelect} color={true} onKeyEvent={idx == 0 ? keydownUpFromMenu : undefined} />)}</>
              : <Box p='3'>
                <Callout.Root color="amber">
                  <Callout.Icon>
                    <InformationCircleIcon width={24} height={24} />
                  </Callout.Icon>
                  <Callout.Text>
                    No nodes were found matching the serach query
                  </Callout.Text>
                </Callout.Root>
              </Box>)
          }
        </ScrollArea>
      </DropdownMenu.Content>
    </DropdownMenu.Root>
  );
}
