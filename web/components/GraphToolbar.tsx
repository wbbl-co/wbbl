import * as Toolbar from "@radix-ui/react-toolbar";
import { Tooltip } from "@radix-ui/themes";
import { CSSProperties, useCallback, useState } from "react";
import CoreLineCursor from "./icons/core-line/CoreLineCursor";
import CoreLineAreaSelection from "./icons/core-line/CoreLineAreaSelection";
import CoreLineComment from "./icons/core-line/CoreLineComment";
import CoreLineZoomIn from "./icons/core-line/CoreLineZoomIn";
import CoreLineZoomOut from "./icons/core-line/CoreLineZoomOut";
import CoreLineResetZoom from "./icons/core-line/CoreLineZoomResetZoom";

const options = ["pointer", "box-select", "comment"] as const;

export default function GraphToolbar() {
  const [[position, prevPosition], setPosition] = useState([0, 0]);

  const onSelect = useCallback(
    (value: string) => {
      const index = options.indexOf(value as (typeof options)[number]) ?? 0;
      if (index >= 0) {
        setPosition([index, position]);
      }
    },
    [setPosition, position],
  );
  const onAnimationEnd = useCallback(() => {
    setPosition([position, position]);
  }, [setPosition, position]);
  return (
    <Toolbar.Root className="ToolbarRoot" aria-label="Formatting options">
      <Toolbar.ToggleGroup
        value={options[position]}
        style={
          {
            "--toggle-index": position,
            "--prev-toggle-index": prevPosition,
          } as CSSProperties
        }
        className={`ToolbarToggleGroup ${prevPosition !== position ? "animating" : ""}`}
        type="single"
        onAnimationEnd={onAnimationEnd}
        aria-label="Text formatting"
        onValueChange={onSelect}
      >
        <Tooltip content={"Cursor"}>
          <Toolbar.ToggleItem
            className="ToolbarToggleItem"
            value="pointer"
            aria-label="Pointer"
          >
            <CoreLineCursor />
          </Toolbar.ToggleItem>
        </Tooltip>
        <Tooltip content={"Box Select"}>
          <Toolbar.ToggleItem
            className="ToolbarToggleItem"
            value="box-select"
            aria-label="Box Selection"
          >
            <CoreLineAreaSelection />
          </Toolbar.ToggleItem>
        </Tooltip>
        <Tooltip content={"Comment"}>
          <Toolbar.ToggleItem
            className="ToolbarToggleItem"
            value="comment"
            aria-label="Comment"
          >
            <CoreLineComment />
          </Toolbar.ToggleItem>
        </Tooltip>
      </Toolbar.ToggleGroup>
      <Toolbar.Separator className="ToolbarSeparator" />
      <Tooltip content={"Zoom In"}>
        <Toolbar.Button className="ToolbarButton">
          <CoreLineZoomIn />
        </Toolbar.Button>
      </Tooltip>
      <Tooltip content={"Zoom Out"}>
        <Toolbar.Button className="ToolbarButton">
          <CoreLineZoomOut />
        </Toolbar.Button>
      </Tooltip>
      <Tooltip content={"Reset Zoom"}>
        <Toolbar.Button className="ToolbarButton">
          <CoreLineResetZoom />
        </Toolbar.Button>
      </Tooltip>
    </Toolbar.Root>
  );
}
