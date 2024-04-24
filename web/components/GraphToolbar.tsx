import * as Toolbar from "@radix-ui/react-toolbar";
import { Tooltip } from "@radix-ui/themes";
import {
  CSSProperties,
  useCallback,
  useContext,
  useEffect,
  useRef,
  useState,
} from "react";
import CoreLineCursor from "./icons/core-line/CoreLineCursor";
import CoreLineAreaSelection from "./icons/core-line/CoreLineAreaSelection";
import CoreLineComment from "./icons/core-line/CoreLineComment";
import CoreLineZoomIn from "./icons/core-line/CoreLineZoomIn";
import CoreLineZoomOut from "./icons/core-line/CoreLineZoomOut";
import CoreLineResetZoom from "./icons/core-line/CoreLineRecenter";
import {
  Viewport,
  useOnViewportChange,
  useReactFlow,
  useViewport,
} from "@xyflow/react";
import { useScopedShortcut } from "../hooks/use-shortcut";
import { KeyboardShortcut } from "../../pkg/wbbl";
import {
  WbblPreferencesStoreContext,
  useKeyBinding,
} from "../hooks/use-preferences-store";
import formatKeybinding from "../utils/format-keybinding";
import { useHotkeys } from "react-hotkeys-hook";

const options = ["pointer", "box-select", "comment"] as const;

export default function GraphToolbar() {
  const flow = useReactFlow();
  const viewport = useViewport();
  const preferencesStore = useContext(WbblPreferencesStoreContext);

  const zoomOutKeybinding = useKeyBinding(
    preferencesStore,
    KeyboardShortcut.ZoomOut,
  );
  const zoomInKeybinding = useKeyBinding(
    preferencesStore,
    KeyboardShortcut.ZoomIn,
  );
  const recenterKeybinding = useKeyBinding(
    preferencesStore,
    KeyboardShortcut.Recenter,
  );

  const selectionKeybinding = useKeyBinding(
    preferencesStore,
    KeyboardShortcut.Selection,
  );

  const commentKeybinding = useKeyBinding(
    preferencesStore,
    KeyboardShortcut.AddComment,
  );

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

  const [modeBeforeTransientSelection, setModeBeforeTransientSelection] =
    useState(0);
  useHotkeys(
    selectionKeybinding ?? "",
    () => {
      setPosition([1, position]);
      setModeBeforeTransientSelection(position);
    },
    { keydown: true, keyup: false },
    [setPosition, position, setModeBeforeTransientSelection],
  );
  useHotkeys(
    selectionKeybinding ?? "",
    () => {
      setPosition([modeBeforeTransientSelection, 1]);
    },
    {
      keydown: false,
      keyup: true,
    },
    [setPosition, modeBeforeTransientSelection],
  );

  useScopedShortcut(
    KeyboardShortcut.AddComment,
    () => {
      setPosition([2, position]);
    },
    [setPosition, position],
  );

  useScopedShortcut(
    KeyboardShortcut.UseCusor,
    () => {
      setPosition([0, position]);
    },
    [setPosition, position],
    { disabled: position == 0 },
  );

  const onButtonClicked = useCallback((button: HTMLButtonElement) => {
    button.dataset.clicked = "true";
  }, []);

  const onButtonAnimationEnd = useCallback<React.AnimationEventHandler>(
    (evt) => {
      const button = evt.currentTarget as HTMLButtonElement;
      button.dataset.clicked = "false";
    },
    [],
  );
  const fitViewRef = useRef<HTMLButtonElement>(null);
  const fitView = useCallback(() => {
    if (fitViewRef.current) {
      flow.fitView({ duration: 300 });
      onButtonClicked(fitViewRef.current);
    }
  }, [flow, onButtonClicked, fitViewRef]);

  const zoomOutRef = useRef<HTMLButtonElement>(null);
  const zoomOut = useCallback(() => {
    if (zoomOutRef.current) {
      flow.zoomOut({ duration: 200 });
      onButtonClicked(zoomOutRef.current);
    }
  }, [flow, zoomOutRef, onButtonClicked]);

  const zoomInRef = useRef<HTMLButtonElement>(null);
  const zoomIn = useCallback(() => {
    if (zoomInRef.current) {
      flow.zoomIn({ duration: 200 });
      onButtonClicked(zoomInRef.current);
    }
  }, [flow, zoomInRef, onButtonClicked]);

  const zoomInDisabled = viewport.zoom >= 1.4;
  const zoomOutDisabled = viewport.zoom <= 0.25;

  useScopedShortcut(KeyboardShortcut.ZoomOut, zoomOut, [zoomOut], {
    disabled: zoomOutDisabled,
  });
  useScopedShortcut(KeyboardShortcut.ZoomIn, zoomIn, [zoomIn], {
    disabled: zoomInDisabled,
  });

  const onViewportChange = useCallback(
    (change: Viewport) => {
      if (change.zoom > viewport.zoom && zoomInRef.current) {
        onButtonClicked(zoomInRef.current);
      } else if (change.zoom < viewport.zoom && zoomOutRef.current) {
        onButtonClicked(zoomOutRef.current);
      }
    },
    [viewport, zoomInRef, zoomOutRef, onButtonClicked],
  );
  useOnViewportChange({
    onChange: onViewportChange,
  });

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
        <Tooltip
          content={`Box Select${selectionKeybinding ? ` (${formatKeybinding(selectionKeybinding)})` : ""}`}
        >
          <Toolbar.ToggleItem
            className="ToolbarToggleItem"
            value="box-select"
            aria-label="Box Selection"
          >
            <CoreLineAreaSelection />
          </Toolbar.ToggleItem>
        </Tooltip>
        <Tooltip
          content={`Comment ${commentKeybinding ? ` (${formatKeybinding(commentKeybinding)})` : ""}`}
        >
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
      <Tooltip
        content={`Zoom In${zoomInKeybinding ? ` (${formatKeybinding(zoomInKeybinding)})` : ""}`}
      >
        <Toolbar.Button
          ref={zoomInRef}
          disabled={zoomInDisabled}
          className="ToolbarButton"
          onAnimationEnd={onButtonAnimationEnd}
          onClick={zoomIn}
        >
          <CoreLineZoomIn />
        </Toolbar.Button>
      </Tooltip>
      <Tooltip
        content={`Zoom Out${zoomOutKeybinding ? ` (${formatKeybinding(zoomOutKeybinding)})` : ""}`}
      >
        <Toolbar.Button
          ref={zoomOutRef}
          disabled={zoomOutDisabled}
          className="ToolbarButton"
          onAnimationEnd={onButtonAnimationEnd}
          onClick={zoomOut}
        >
          <CoreLineZoomOut />
        </Toolbar.Button>
      </Tooltip>
      <Tooltip
        content={`Recenter${recenterKeybinding ? ` (${formatKeybinding(recenterKeybinding)})` : ""}`}
      >
        <Toolbar.Button
          ref={fitViewRef}
          className="ToolbarButton"
          onAnimationEnd={onButtonAnimationEnd}
          onClick={fitView}
        >
          <CoreLineResetZoom />
        </Toolbar.Button>
      </Tooltip>
    </Toolbar.Root>
  );
}
