import { createRootRoute, Outlet } from '@tanstack/react-router'
import { Theme } from "@radix-ui/themes";
import { AvailableActions, AvailableActionsContext } from '../hooks/use-actions-menu';
import { ShortcutScope } from '../hooks/use-shortcut';
import { HotkeysProvider } from 'react-hotkeys-hook';
import { useThemePreferences, WbblPreferencesStoreContext } from '../hooks/use-preferences-store';
import { useCallback, useMemo, useRef, useState, MouseEvent as ReactMouseEvent } from 'react';
import { BaseTheme, WbblWebappPreferencesStore } from "../../pkg/wbbl";
import { ApplicationMenuContext } from '../components/ApplicationMenu';
import SearchMenu from '../components/SearchMenu';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';

const queryClient = new QueryClient()

export const Route = createRootRoute({
    component: () => {
        const preferencesStore = useMemo(
            () => WbblWebappPreferencesStore.empty(),
            [],
        );
        const { currentTheme } = useThemePreferences(preferencesStore);
        const availableActionsContext: AvailableActions = useMemo(
            () => ({ actions: new Map() }),
            [],
        );
        const mousePosition = useRef({
            x: window.innerWidth / 2,
            y: window.innerHeight / 2,
        });
        const setMousePositionCallback = useCallback(
            (evt: ReactMouseEvent<HTMLDivElement>) => {
                mousePosition.current = { x: evt.clientX, y: evt.clientY };
            },
            [mousePosition],
        );
        const [actionMenuSettings, setActionMenuSettings] = useState({
            open: false,
            useMousePosition: false,
        });

        return (
            <QueryClientProvider client={queryClient}>
                <AvailableActionsContext.Provider value={availableActionsContext} >
                    <ShortcutScope scope="root">
                        <HotkeysProvider initiallyActiveScopes={["root"]}>
                            <WbblPreferencesStoreContext.Provider value={preferencesStore}>
                                <Theme
                                    appearance={currentTheme == BaseTheme.Dark ? "dark" : "light"}
                                    accentColor="lime"
                                    grayColor="gray"
                                    onMouseMove={setMousePositionCallback}
                                >
                                    <div style={{ height: "100dvh", width: "100dvw" }}>

                                        <ApplicationMenuContext.Provider
                                            value={{
                                                showNodesInActionMenu: !!availableActionsContext.addNode,
                                                setActionMenuSettings: setActionMenuSettings
                                            }}>
                                            <Outlet />
                                        </ApplicationMenuContext.Provider>
                                        <SearchMenu
                                            useMousePosition={actionMenuSettings.useMousePosition}
                                            mousePosition={mousePosition}
                                            open={actionMenuSettings.open}
                                            setActionMenuSettings={setActionMenuSettings}
                                        />
                                    </div>

                                </Theme>
                            </WbblPreferencesStoreContext.Provider>
                        </HotkeysProvider>
                    </ShortcutScope>
                </AvailableActionsContext.Provider >
            </QueryClientProvider>)

    },
})