import type { Terminal } from "@xterm/xterm";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  type ActionLink,
  ActionLinksAddon,
  type ActionLinksAddonOptions,
  type ResolvedAction,
} from "../lib/actionLinksAddon";
import {
  createArchiveMatcher,
  createHostPortMatcher,
  createIPv4Matcher,
} from "../lib/actionLinksMatcher";
import type { AppSettings } from "../types/global";

const DEFAULT_ACTION_LINK_MATCHERS = {
  ipv4: true,
  archive: true,
  host_port: true,
} as const;

export interface TooltipState {
  x: number;
  y: number;
  link: ActionLink;
}

export interface MenuState {
  x: number;
  y: number;
  link: ActionLink;
  actions: ResolvedAction[];
  prepare: (actionId: string) => void;
}

export interface UseActionLinksResult {
  tooltipState: TooltipState | null;
  menuState: MenuState | null;
  closeMenu: () => void;
  closeTooltip: () => void;
}

/**
 * Creates and manages an ActionLinksAddon tied to the terminal session lifecycle.
 * Returns reactive tooltip/menu state for overlay rendering.
 */
export function useActionLinks(
  terminal: Terminal | null,
  terminalSettings: AppSettings["terminal"],
  _sessionId: string,
  prepareCommandRef: React.RefObject<((command: string) => void) | null>,
  suspended = false,
): UseActionLinksResult {
  const addonRef = useRef<ActionLinksAddon | null>(null);
  const [addonInstance, setAddonInstance] = useState<ActionLinksAddon | null>(null);
  const [tooltipState, setTooltipState] = useState<TooltipState | null>(null);
  const [menuState, setMenuState] = useState<MenuState | null>(null);

  const closeMenu = useCallback(() => setMenuState(null), []);
  const closeTooltip = useCallback(() => setTooltipState(null), []);

  const matcherSettings = terminalSettings.action_links_matchers ?? DEFAULT_ACTION_LINK_MATCHERS;
  const enabled = terminalSettings.action_links_enabled ?? false;

  const matchers = useMemo(() => {
    const list = [];
    if (matcherSettings.host_port) list.push(createHostPortMatcher());
    if (matcherSettings.ipv4) list.push(createIPv4Matcher());
    if (matcherSettings.archive) list.push(createArchiveMatcher());
    return list;
  }, [matcherSettings?.ipv4, matcherSettings?.archive, matcherSettings?.host_port]);

  // Create and load addon once per terminal instance.
  useEffect(() => {
    if (!enabled) {
      addonRef.current?.dispose();
      addonRef.current = null;
      setAddonInstance(null);
      setTooltipState(null);
      setMenuState(null);
      return;
    }

    if (!terminal) return;

    const options: ActionLinksAddonOptions = {
      allowCtrlOrMetaClickExecute: true,
      allowAltClickMenu: true,
      fallbackAltClickToDefaultAction: true,
      executeCommand: (command) => prepareCommandRef.current?.(command),
      showTooltip: ({ event, link }) => {
        setTooltipState({ x: event.clientX, y: event.clientY, link });
      },
      hideTooltip: () => setTooltipState(null),
      showMenu: ({ event, link, actions, execute }) => {
        setMenuState({ x: event.clientX, y: event.clientY, link, actions, prepare: execute });
      },
    };

    const addon = new ActionLinksAddon([], options);
    terminal.loadAddon(addon);
    addonRef.current = addon;
    setAddonInstance(addon);

    return () => {
      addon.dispose();
      addonRef.current = null;
      setAddonInstance((current) => (current === addon ? null : current));
      setTooltipState(null);
      setMenuState(null);
    };
  }, [terminal, enabled, prepareCommandRef]);

  // Sync matchers and enabled state when settings change
  useEffect(() => {
    if (!addonInstance) return;
    addonInstance.setSuspended(suspended);
    if (!suspended) {
      addonInstance.setMatchers(matchers);
    }
    if (suspended || matchers.length === 0) {
      setTooltipState(null);
      setMenuState(null);
    }
  }, [addonInstance, matchers, suspended]);

  return { tooltipState, menuState, closeMenu, closeTooltip };
}
