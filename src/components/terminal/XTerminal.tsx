import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { FitAddon } from "@xterm/addon-fit";
import { WebLinksAddon } from "@xterm/addon-web-links";
import { type IMarker, Terminal } from "@xterm/xterm";
import { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { useTheme } from "../../context/ThemeContext";
import type { FuzzyResult } from "../../types";
import CommandSuggestions from "./CommandSuggestions";
import "@xterm/xterm/css/xterm.css";

/** Shell integration state tracked via OSC 133 sequences. */
interface ShellIntegrationState {
  /** Whether the shell emits OSC 133 sequences. Set to true on first receipt. */
  enabled: boolean;
  /** Marker at the beginning of the prompt (OSC 133;A). */
  promptStartMarker: IMarker | null;
  /** Marker at the end of the prompt / start of user command input (OSC 133;B). */
  commandStartMarker: IMarker | null;
  /** CursorX at the moment OSC 133;B was received. */
  commandStartX: number;
  /** Fallback: promptEndX detected heuristically (for shells without OSC 133). */
  fallbackPromptEndX: number;
  /** Fallback: whether we still need to detect the prompt position. */
  fallbackNeedsDetection: boolean;
}

interface XTerminalProps {
  sessionId: string;
  active: boolean;
}

/**
 * xterm.js terminal for a session. Handles OSC 133 shell integration (or fallback prompt
 * detection), fuzzy command history suggestions, and resize/fit. Key props: sessionId, active.
 */
export default function XTerminal({ sessionId, active }: XTerminalProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const terminalRef = useRef<Terminal | null>(null);
  const fitAddonRef = useRef<FitAddon | null>(null);
  const { theme } = useTheme();
  const { t } = useTranslation();

  // Fuzzy search: refs for the onData callback, state for rendering
  const currentLineRef = useRef("");
  const suggestionsRef = useRef<FuzzyResult[]>([]);
  const selectedIndexRef = useRef(-1);
  const showSuggestionsRef = useRef(false);
  const searchTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Shell integration: OSC 133 based prompt/command tracking with fallback
  const shellIntegrationRef = useRef<ShellIntegrationState>({
    enabled: false,
    promptStartMarker: null,
    commandStartMarker: null,
    commandStartX: 0,
    fallbackPromptEndX: 0,
    fallbackNeedsDetection: true,
  });

  const [suggestions, setSuggestions] = useState<FuzzyResult[]>([]);
  const [selectedIndex, setSelectedIndex] = useState(-1);
  const [showSuggestions, setShowSuggestions] = useState(false);
  const [cursorPosition, setCursorPosition] = useState({ top: 0, left: 0 });

  /**
   * Read command text from the terminal buffer.
   * Uses marker-based reading when shell integration (OSC 133) is active,
   * falls back to heuristic prompt-end position otherwise.
   */
  const readCommandFromBuffer = (): string => {
    const terminal = terminalRef.current;
    if (!terminal) return currentLineRef.current;
    const si = shellIntegrationRef.current;

    try {
      const buf = terminal.buffer.active;

      if (si.enabled && si.commandStartMarker) {
        // OSC 133 path: read between the B-marker and the current cursor
        return readBetweenMarkerAndCursor(terminal, si.commandStartMarker, si.commandStartX);
      }

      // Fallback path: single-line read from heuristic prompt end
      const row = buf.baseY + buf.cursorY;
      const line = buf.getLine(row);
      if (!line) return currentLineRef.current;
      return line.translateToString(true, si.fallbackPromptEndX);
    } catch {
      return currentLineRef.current;
    }
  };

  /**
   * Read text from the buffer between a marker position and the current cursor.
   * Supports multi-line commands (wrapped or explicit newlines).
   */
  const readBetweenMarkerAndCursor = (
    terminal: Terminal,
    startMarker: IMarker,
    startX: number,
  ): string => {
    try {
      const buf = terminal.buffer.active;
      const startRow = startMarker.line;
      const endRow = buf.baseY + buf.cursorY;

      if (startRow === endRow) {
        const line = buf.getLine(startRow);
        return line?.translateToString(true, startX) ?? "";
      }

      // Multi-line command: concatenate rows
      let result = "";
      for (let row = startRow; row <= endRow; row++) {
        const line = buf.getLine(row);
        if (!line) continue;
        if (row === startRow) {
          result += line.translateToString(true, startX);
        } else if (line.isWrapped) {
          // Soft-wrapped continuation — no separator
          result += line.translateToString(true);
        } else {
          result += `\n${line.translateToString(true)}`;
        }
      }
      return result;
    } catch {
      return "";
    }
  };

  // Accept a suggestion from the overlay (click) — erase + write + execute
  const handleSelectSuggestion = useCallback(
    (command: string) => {
      const actualCmd = readCommandFromBuffer();
      const eraseChars = "\x7f".repeat(actualCmd.length);
      invoke("write_to_session", {
        sessionId,
        data: `${eraseChars + command}\r`,
      }).catch(() => {});
      invoke("add_command_history", { sessionId, command }).catch(() => {});
      currentLineRef.current = "";
      shellIntegrationRef.current.fallbackNeedsDetection = true;

      showSuggestionsRef.current = false;
      suggestionsRef.current = [];
      selectedIndexRef.current = -1;
      setShowSuggestions(false);
      setSuggestions([]);
      setSelectedIndex(-1);

      terminalRef.current?.focus();
    },
    // biome-ignore lint/correctness/useExhaustiveDependencies: readCommandFromBuffer reads refs directly, not state; adding it would recreate the callback on every render
    [sessionId],
  );

  // Dismiss suggestions from the overlay
  const handleDismissSuggestions = useCallback(() => {
    showSuggestionsRef.current = false;
    suggestionsRef.current = [];
    selectedIndexRef.current = -1;
    setShowSuggestions(false);
    setSuggestions([]);
    setSelectedIndex(-1);

    terminalRef.current?.focus();
  }, []);

  // Create and setup terminal
  useEffect(() => {
    if (!containerRef.current) return;

    const terminal = new Terminal({
      cursorBlink: true,
      cursorStyle: "block",
      fontSize: 14,
      fontFamily: "'JetBrains Mono', monospace",
      theme: { ...theme.colors.terminal },
      allowProposedApi: true,
    });

    const fitAddon = new FitAddon();
    const webLinksAddon = new WebLinksAddon();

    terminal.loadAddon(fitAddon);
    terminal.loadAddon(webLinksAddon);
    terminal.open(containerRef.current);

    // Initial fit
    requestAnimationFrame(() => {
      fitAddon.fit();
    });

    terminalRef.current = terminal;
    fitAddonRef.current = fitAddon;

    // ── Fuzzy search helpers ──────────────────────────────────────────────

    /** Get the cursor position in viewport (fixed) coordinates. */
    const getCursorViewportPosition = (): { top: number; left: number } => {
      try {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        const core = (terminal as any)._core;
        const dims = core._renderService.dimensions;
        const cellHeight: number = dims.css.cell.height;
        const cellWidth: number = dims.css.cell.width;

        const cursorY = terminal.buffer.active.cursorY;
        const cursorX = terminal.buffer.active.cursorX;

        // Get the xterm-screen element's viewport rect for accurate offset
        const screenEl = terminal.element?.querySelector(".xterm-screen");
        if (!screenEl) return { top: 0, left: 0 };

        const screenRect = screenEl.getBoundingClientRect();

        return {
          top: screenRect.top + (cursorY + 1) * cellHeight,
          left: screenRect.left + cursorX * cellWidth,
        };
      } catch {
        return { top: 0, left: 0 };
      }
    };

    const dismissSuggestions = () => {
      showSuggestionsRef.current = false;
      suggestionsRef.current = [];
      selectedIndexRef.current = -1;
      setSuggestions([]);
      setSelectedIndex(-1);
      setShowSuggestions(false);
    };

    /** Read the command from the terminal buffer (effect-local version). */
    const readBufferCommand = (): string => {
      const si = shellIntegrationRef.current;
      try {
        if (si.enabled && si.commandStartMarker) {
          return readBetweenMarkerAndCursor(terminal, si.commandStartMarker, si.commandStartX);
        }
        // Fallback: single-line read from heuristic prompt end
        const buf = terminal.buffer.active;
        const row = buf.baseY + buf.cursorY;
        const line = buf.getLine(row);
        if (!line) return currentLineRef.current;
        return line.translateToString(true, si.fallbackPromptEndX);
      } catch {
        return currentLineRef.current;
      }
    };

    const triggerSearch = () => {
      if (searchTimerRef.current) clearTimeout(searchTimerRef.current);

      // Guard: only search if the user has typed something since the last
      // reset.  Use length (not trim) so that even a single space after an
      // arrow-key recall still triggers a buffer read.
      if (currentLineRef.current.length === 0) {
        dismissSuggestions();
        return;
      }

      searchTimerRef.current = setTimeout(async () => {
        // Read the real command from the terminal buffer — by now (80 ms
        // later) the echo has arrived, so the buffer includes shell
        // completions, IME commits, etc.
        const pattern = readBufferCommand();
        if (!pattern.trim()) {
          dismissSuggestions();
          return;
        }
        try {
          const results = await invoke<FuzzyResult[]>("fuzzy_search_history", {
            pattern,
            limit: 8,
          });
          suggestionsRef.current = results;
          selectedIndexRef.current = -1;
          showSuggestionsRef.current = results.length > 0;
          setSuggestions(results);
          setSelectedIndex(-1);
          setShowSuggestions(results.length > 0);

          // Capture cursor position after echo has arrived
          if (results.length > 0) {
            setCursorPosition(getCursorViewportPosition());
          }
        } catch {
          // Ignore errors
        }
      }, 80);
    };

    // ── OSC 133 Shell Integration handlers ─────────────────────────────
    //
    // These hook into the FinalTerm / VS Code shell integration protocol.
    // The shell emits OSC 133 sequences to mark prompt boundaries and
    // command execution, giving us precise command capture without
    // keystroke-tracking heuristics.

    const oscDisposable = terminal.parser.registerOscHandler(133, (data) => {
      const si = shellIntegrationRef.current;

      if (data.startsWith("A")) {
        // ── Prompt Start ──────────────────────────────────────────────
        si.enabled = true;
        si.promptStartMarker?.dispose();
        si.promptStartMarker = terminal.registerMarker(0);
        return false;
      }

      if (data.startsWith("B")) {
        // ── Prompt End / Command Input Start ──────────────────────────
        si.enabled = true;
        si.commandStartMarker?.dispose();
        si.commandStartMarker = terminal.registerMarker(0);
        si.commandStartX = terminal.buffer.active.cursorX;
        return false;
      }

      if (data.startsWith("C")) {
        // ── Command Executed (user pressed Enter) ─────────────────────
        si.enabled = true;
        if (si.commandStartMarker) {
          const command = readBetweenMarkerAndCursor(
            terminal,
            si.commandStartMarker,
            si.commandStartX,
          ).trim();
          if (command) {
            invoke("add_command_history", { sessionId, command }).catch(() => {});
          }
        }
        currentLineRef.current = "";
        dismissSuggestions();
        return false;
      }

      if (data.startsWith("D")) {
        // ── Command Finished ──────────────────────────────────────────
        // Optionally: const exitCode = data.split(";")[1];
        si.enabled = true;
        return false;
      }

      return false;
    });

    // ── Fallback prompt detection via onWriteParsed ──────────────────
    //
    // For shells that do NOT emit OSC 133, we use onWriteParsed (fires
    // after terminal.write data is fully parsed, at most once per frame)
    // to continuously track the cursor position while detection is
    // pending.  The value is only finalized (fallbackNeedsDetection set
    // to false) by the keystroke handler in onData — this avoids a race
    // where the debounce timer fires between the last \n and the prompt
    // text, locking in cursorX=0.

    const writeParsedDisposable = terminal.onWriteParsed(() => {
      const si = shellIntegrationRef.current;
      if (si.enabled) return; // OSC 133 is handling everything

      // Keep updating the prompt-end position as output settles.
      // Do NOT set fallbackNeedsDetection=false here — the keystroke
      // handler in onData finalizes it when the user actually starts
      // typing, which guarantees the cursor is past the full prompt.
      if (si.fallbackNeedsDetection) {
        si.fallbackPromptEndX = terminal.buffer.active.cursorX;
      }
    });

    // ── Backend output & session listeners ───────────────────────────────

    let outputUnlisten: UnlistenFn | null = null;
    let closedUnlisten: UnlistenFn | null = null;

    const setupListeners = async () => {
      outputUnlisten = await listen<string>(`terminal-output-${sessionId}`, (event) => {
        terminal.write(event.payload);

        // When backend output contains a newline, reset the keystroke
        // buffer so stale characters don't leak into fuzzy search.
        // Prompt detection is now handled by onWriteParsed (fallback)
        // or OSC 133;B (shell integration).
        if (event.payload.includes("\n")) {
          const si = shellIntegrationRef.current;
          currentLineRef.current = "";
          if (!si.enabled) {
            si.fallbackNeedsDetection = true;
          }
          dismissSuggestions();
        }
      });

      closedUnlisten = await listen<void>(`session-closed-${sessionId}`, () => {
        terminal.write(`\r\n\x1b[31m[${t("terminal.sessionDisconnected")}]\x1b[0m\r\n`);
      });

      // Listener is ready — tell the backend to flush buffered output
      await invoke("attach_session", { sessionId });
    };
    setupListeners();

    // ── Handle user input ─────────────────────────────────────────────────

    const dataDisposable = terminal.onData((data) => {
      // When suggestions are visible, intercept navigation keys
      if (showSuggestionsRef.current && suggestionsRef.current.length > 0) {
        // Tab → accept selected suggestion (fill in without executing)
        // Only when an item is actually selected
        if (data === "\t" && selectedIndexRef.current >= 0) {
          const selected = suggestionsRef.current[selectedIndexRef.current];
          if (selected) {
            const actualCmd = readBufferCommand();
            const eraseChars = "\x7f".repeat(actualCmd.length);
            invoke("write_to_session", {
              sessionId,
              data: eraseChars + selected.command,
            }).catch(() => {});
            currentLineRef.current = selected.command;
            dismissSuggestions();
          }
          return;
        }

        // Up arrow → navigate up; from -1 (no selection) goes to last item
        if (data === "\x1b[A") {
          const cur = selectedIndexRef.current;
          const newIdx = cur === -1 ? suggestionsRef.current.length - 1 : cur === 0 ? -1 : cur - 1;
          selectedIndexRef.current = newIdx;
          setSelectedIndex(newIdx);
          return;
        }

        // Down arrow → navigate down; from -1 (no selection) goes to first item
        if (data === "\x1b[B") {
          const cur = selectedIndexRef.current;
          const newIdx = cur === -1 ? 0 : cur === suggestionsRef.current.length - 1 ? -1 : cur + 1;
          selectedIndexRef.current = newIdx;
          setSelectedIndex(newIdx);
          return;
        }

        // Escape → dismiss suggestions
        if (data === "\x1b") {
          dismissSuggestions();
          return;
        }

        // Enter → if an item is selected, accept + execute it
        if (data === "\r" && selectedIndexRef.current >= 0) {
          const selected = suggestionsRef.current[selectedIndexRef.current];
          if (selected) {
            const actualCmd = readBufferCommand();
            const eraseChars = "\x7f".repeat(actualCmd.length);
            invoke("write_to_session", {
              sessionId,
              data: `${eraseChars + selected.command}\r`,
            }).catch(() => {});
            // When shell integration is active, OSC 133;C will capture
            // the command for history. In fallback mode, record it here.
            if (!shellIntegrationRef.current.enabled) {
              invoke("add_command_history", {
                sessionId,
                command: selected.command,
              }).catch(() => {});
            }
            currentLineRef.current = "";
            shellIntegrationRef.current.fallbackNeedsDetection = true;
            dismissSuggestions();
          }
          return;
        }
      }

      // ── Input buffering & history ───────────────────────────────────
      const si = shellIntegrationRef.current;

      if (data === "\r") {
        // On Enter: in fallback mode, read command from buffer for history.
        // When OSC 133 is active, the C-handler captures the command instead.
        if (!si.enabled) {
          const bufCmd = readBufferCommand().trim();
          const cmd = bufCmd || currentLineRef.current.trim();
          if (cmd) {
            invoke("add_command_history", { sessionId, command: cmd });
          }
        }
        currentLineRef.current = "";
        si.fallbackNeedsDetection = true;
        dismissSuggestions();
      } else if (data === "\u007f" || data === "\b") {
        // Backspace — update keystroke buffer for fuzzy search
        currentLineRef.current = currentLineRef.current.slice(0, -1);
        triggerSearch();
      } else if (data === "\t") {
        // Tab — pass through to shell for its own completion.
        triggerSearch();
      } else if (!/[\x00-\x1f\x7f]/.test(data)) {
        // Printable characters
        // Fallback prompt detection: on first keystroke, snapshot cursorX
        if (!si.enabled && si.fallbackNeedsDetection) {
          si.fallbackPromptEndX = terminal.buffer.active.cursorX;
          si.fallbackNeedsDetection = false;
        }
        currentLineRef.current += data;
        triggerSearch();
      } else if (data.startsWith("\x1b")) {
        // Escape sequences (arrow keys, etc.) — invalidate keystroke buffer
        if (!si.enabled && si.fallbackNeedsDetection) {
          si.fallbackPromptEndX = terminal.buffer.active.cursorX;
          si.fallbackNeedsDetection = false;
        }
        currentLineRef.current = "";
        dismissSuggestions();
      } else {
        // Other control characters (Ctrl+C, Ctrl+U, etc.)
        currentLineRef.current = "";
        si.fallbackNeedsDetection = true;
        dismissSuggestions();
      }

      invoke("write_to_session", { sessionId, data }).catch(() => {
        // Session might be closed
      });
    });

    // Send resize events to backend
    const resizeDisposable = terminal.onResize(({ cols, rows }) => {
      invoke("resize_session", { sessionId, cols, rows }).catch(() => {
        // Session might be closed
      });
    });

    // Observe container size changes
    const observer = new ResizeObserver(() => {
      requestAnimationFrame(() => {
        fitAddon.fit();
      });
    });
    observer.observe(containerRef.current);

    return () => {
      if (searchTimerRef.current) clearTimeout(searchTimerRef.current);

      // Dispose shell integration markers
      const si = shellIntegrationRef.current;
      si.promptStartMarker?.dispose();
      si.commandStartMarker?.dispose();
      si.promptStartMarker = null;
      si.commandStartMarker = null;

      // Dispose xterm event / parser handlers
      oscDisposable.dispose();
      writeParsedDisposable.dispose();
      dataDisposable.dispose();
      resizeDisposable.dispose();

      observer.disconnect();
      if (outputUnlisten) outputUnlisten();
      if (closedUnlisten) closedUnlisten();
      terminal.dispose();
      terminalRef.current = null;
      fitAddonRef.current = null;
    };
  }, [sessionId]);

  // Re-fit and focus when tab becomes active
  useEffect(() => {
    if (active && fitAddonRef.current && terminalRef.current) {
      requestAnimationFrame(() => {
        fitAddonRef.current?.fit();
        terminalRef.current?.focus();
      });
    }
  }, [active]);

  // React to theme changes: update terminal colors dynamically
  useEffect(() => {
    if (terminalRef.current) {
      terminalRef.current.options.theme = { ...theme.colors.terminal };
    }
  }, [theme]);

  return (
    <div className="h-full w-full relative" style={{ display: active ? "block" : "none" }}>
      <div ref={containerRef} className="h-full w-full" />
      <CommandSuggestions
        suggestions={suggestions}
        visible={showSuggestions}
        selectedIndex={selectedIndex}
        cursorPosition={cursorPosition}
        onSelect={handleSelectSuggestion}
        onDismiss={handleDismissSuggestions}
      />
    </div>
  );
}
