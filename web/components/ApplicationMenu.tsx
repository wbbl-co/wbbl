import { DropdownMenu, Button, Dialog } from "@radix-ui/themes";
import WbblLogo from "./WbblLogo";
import {
  FunctionComponent,
  useCallback,
  useContext,
  useMemo,
  useState,
} from "react";
import { HomeIcon } from "@heroicons/react/24/solid";
import { MagnifyingGlassIcon } from "@heroicons/react/24/solid";
import { AdjustmentsHorizontalIcon } from "@heroicons/react/24/solid";
import KeybindingDialogContents from "./KeybindingDialogContents";
import React from "react";
import {
  WbblPreferencesStoreContext,
  useThemePreferences,
} from "../hooks/use-preferences-store";
import { BaseTheme } from "../../pkg/wbbl";

export default function ApplicationMenu() {
  const goHome = useCallback(() => {
    window.location.assign("/");
  }, []);

  const [currentDialog, setCurrentDialog] = useState<
    [string, FunctionComponent<{}>] | null
  >(null);
  const setKeybindingDialog = useCallback(() => {
    setCurrentDialog(() => [
      "Key Bindings",
      () => <KeybindingDialogContents />,
    ]);
  }, [setCurrentDialog]);
  const preferencesStore = useContext(WbblPreferencesStoreContext);
  const setMode = useCallback(
    (mode: string) => {
      console.log("Setting Mode", mode);
      try {
        if (mode === "light") {
          preferencesStore.set_base_theme(BaseTheme.Light);
        } else if (mode === "dark") {
          preferencesStore.set_base_theme(BaseTheme.Dark);
        } else if (mode === "system") {
          preferencesStore.set_base_theme(BaseTheme.System);
        }
      } catch (e) {
        console.error(e);
      }
    },
    [preferencesStore],
  );

  const { baseTheme } = useThemePreferences(preferencesStore);
  const themeValue = useMemo(() => {
    if (baseTheme === BaseTheme.Light) {
      return "light";
    } else if (baseTheme === BaseTheme.Dark) {
      return "dark";
    } else {
      return "system";
    }
  }, [baseTheme]);

  return (
    <Dialog.Root>
      <DropdownMenu.Root>
        <DropdownMenu.Trigger className="application-menu-trigger">
          <Button
            size={"3"}
            variant="solid"
            aria-label="Global Application Menu"
          >
            <WbblLogo className="logo-button" />
          </Button>
        </DropdownMenu.Trigger>
        <DropdownMenu.Content>
          <DropdownMenu.Item onClick={goHome}>
            <HomeIcon width={"1em"} /> Home
          </DropdownMenu.Item>
          <DropdownMenu.Item shortcut="␣">
            <MagnifyingGlassIcon width={"1em"} /> Quick Actions
          </DropdownMenu.Item>
          <DropdownMenu.Separator />
          <DropdownMenu.Sub>
            <DropdownMenu.SubTrigger>
              <AdjustmentsHorizontalIcon width={"1em"} /> Preferences
            </DropdownMenu.SubTrigger>
            <DropdownMenu.SubContent>
              <Dialog.Trigger>
                <DropdownMenu.Item onClick={setKeybindingDialog} shortcut="⌘ P">
                  Keyboard Shortcuts
                </DropdownMenu.Item>
              </Dialog.Trigger>
              <DropdownMenu.Separator />
              <DropdownMenu.RadioGroup
                onValueChange={setMode}
                value={themeValue}
              >
                <DropdownMenu.RadioItem value="dark">
                  Dark Theme
                </DropdownMenu.RadioItem>
                <DropdownMenu.RadioItem value="light">
                  Light Theme
                </DropdownMenu.RadioItem>
                <DropdownMenu.RadioItem value="system">
                  System Theme
                </DropdownMenu.RadioItem>
              </DropdownMenu.RadioGroup>
            </DropdownMenu.SubContent>
          </DropdownMenu.Sub>
        </DropdownMenu.Content>
      </DropdownMenu.Root>
      <Dialog.Content title={(currentDialog ?? undefined) && currentDialog![0]}>
        {currentDialog && React.createElement(currentDialog[1], {})}
      </Dialog.Content>
    </Dialog.Root>
  );
}
