import * as Toolbar from "@radix-ui/react-toolbar";
import { Tooltip } from "@radix-ui/themes";
import {
  CSSProperties,
  memo,
  useCallback,
  useContext,
  useRef,
  useState,
} from "react";
import CoreLineCursor from "./icons/core-line/CoreLineCursor";
import CoreLineAreaSelection from "./icons/core-line/CoreLineAreaSelection";
import CoreLineComment from "./icons/core-line/CoreLineComment";
import CoreLineZoomIn from "./icons/core-line/CoreLineZoomIn";
import CoreLineZoomOut from "./icons/core-line/CoreLineZoomOut";
import CoreLineResetZoom from "./icons/core-line/CoreLineRecenter";
import { useReactFlow, useViewport } from "@xyflow/react";
import { useScopedShortcut } from "../hooks/use-shortcut";
import { KeyboardShortcut } from "../../pkg/wbbl";
import {
  WbblPreferencesStoreContext,
  useKeyBinding,
} from "../hooks/use-preferences-store";
import formatKeybinding from "../utils/format-keybinding";
import { useHotkeys } from "react-hotkeys-hook";

export const modes = ["pointer", "box-select", "comment"] as const;

export type GraphToolbarProps = {
  setMode: (value: (typeof modes)[number]) => void;
};

function GraphToolbar({ setMode }: GraphToolbarProps) {
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

  const useCursorKeybinding = useKeyBinding(
    preferencesStore,
    KeyboardShortcut.UseCusor,
  );

  const [[position, prevPosition], setPosition] = useState([0, 0]);

  const onSelect = useCallback(
    (value: string) => {
      const mode = value as (typeof modes)[number];
      const index = modes.indexOf(mode) ?? 0;
      if (index >= 0) {
        setMode(mode);
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
      setMode("comment");
      setPosition([2, position]);
    },
    [setPosition, position],
  );

  useScopedShortcut(
    KeyboardShortcut.UseCusor,
    () => {
      setPosition([0, position]);
      setMode("pointer");
    },
    [setPosition, position],
    { disabled: position == 0 },
  );

  const onButtonClicked = useCallback((button: HTMLButtonElement) => {
    if (!button.hasAttribute("data-clicked")) {
      button.setAttribute("data-clicked", "true");
    }
  }, []);

  const onButtonAnimationEnd = useCallback<React.AnimationEventHandler>(
    (evt) => {
      const button = evt.currentTarget as HTMLButtonElement;
      button.removeAttribute("data-clicked");
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
      onButtonClicked(zoomInRef.current);
      flow.zoomIn({ duration: 200 });
    }
  }, [flow, zoomInRef, onButtonClicked]);

  const zoomInDisabled = viewport.zoom >= 1.4;
  const zoomOutDisabled = viewport.zoom <= 0.25;

  useScopedShortcut(
    KeyboardShortcut.ZoomOut,
    zoomOut,
    [zoomOut, zoomOutDisabled],
    {
      disabled: zoomOutDisabled,
    },
  );
  useScopedShortcut(
    KeyboardShortcut.ZoomIn,
    zoomIn,
    [zoomIn, zoomOutDisabled],
    {
      disabled: zoomInDisabled,
    },
  );

  return (
    <Toolbar.Root className="ToolbarRoot" aria-label="Formatting options">
      <Toolbar.ToggleGroup
        value={modes[position]}
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
        <Tooltip
          content={`Cursor${useCursorKeybinding ? ` (${formatKeybinding(useCursorKeybinding)})` : ""}`}
        >
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

export default memo(GraphToolbar);
