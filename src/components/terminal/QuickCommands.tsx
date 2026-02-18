import { invoke } from "@tauri-apps/api/core";
import { memo, useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import type { QuickCommand } from "../../types";

interface QuickCommandsProps {
  onSend: (command: string) => void;
}

/** Editable quick-command buttons. Loads/saves from backend; onSend runs command in active tab. */
function QuickCommands({ onSend }: QuickCommandsProps) {
  const { t } = useTranslation();
  const [commands, setCommands] = useState<QuickCommand[]>([]);
  const [adding, setAdding] = useState(false);
  const [newLabel, setNewLabel] = useState("");
  const [newCommand, setNewCommand] = useState("");
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editLabel, setEditLabel] = useState("");
  const [editCommand, setEditCommand] = useState("");
  const labelInputRef = useRef<HTMLInputElement>(null);
  const editLabelRef = useRef<HTMLInputElement>(null);
  const loaded = useRef(false);
  const saveTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Load from backend on mount
  useEffect(() => {
    invoke<QuickCommand[]>("get_quick_commands")
      .then((cmds) => {
        setCommands(cmds);
        loaded.current = true;
      })
      .catch(() => {
        loaded.current = true;
      });
  }, []);

  // Debounced save to backend on change
  useEffect(() => {
    if (!loaded.current) return;
    if (saveTimerRef.current) clearTimeout(saveTimerRef.current);
    saveTimerRef.current = setTimeout(() => {
      invoke("save_quick_commands", { commands }).catch(() => {});
    }, 300);
    return () => {
      if (saveTimerRef.current) clearTimeout(saveTimerRef.current);
    };
  }, [commands]);

  useEffect(() => {
    if (adding) labelInputRef.current?.focus();
  }, [adding]);

  useEffect(() => {
    if (editingId) editLabelRef.current?.focus();
  }, [editingId]);

  const handleAdd = useCallback(() => {
    const label = newLabel.trim();
    const cmd = newCommand.trim();
    if (!label || !cmd) return;
    setCommands((prev) => [...prev, { id: `qc-${Date.now()}`, label, command: cmd }]);
    setNewLabel("");
    setNewCommand("");
    setAdding(false);
  }, [newLabel, newCommand]);

  const handleDelete = useCallback((id: string) => {
    setCommands((prev) => prev.filter((c) => c.id !== id));
  }, []);

  const startEdit = useCallback((cmd: QuickCommand) => {
    setEditingId(cmd.id);
    setEditLabel(cmd.label);
    setEditCommand(cmd.command);
  }, []);

  const handleEditSave = useCallback(() => {
    const label = editLabel.trim();
    const cmd = editCommand.trim();
    if (!label || !cmd || !editingId) {
      setEditingId(null);
      return;
    }
    setCommands((prev) =>
      prev.map((c) => (c.id === editingId ? { ...c, label, command: cmd } : c)),
    );
    setEditingId(null);
  }, [editingId, editLabel, editCommand]);

  return (
    <div className="h-full" style={{ backgroundColor: "var(--df-bg-panel)" }}>
      <div className="flex items-center gap-1 px-2 py-1.5 h-full overflow-x-auto overflow-y-auto terminal-scroll flex-wrap content-start">
        <span
          className="material-icons text-sm shrink-0 mr-0.5"
          style={{ color: "var(--df-text-dimmed)" }}
        >
          bolt
        </span>

        {commands.map((cmd) =>
          editingId === cmd.id ? (
            <div
              key={cmd.id}
              className="flex items-center gap-1 rounded px-1.5 py-0.5 shrink-0"
              style={{ backgroundColor: "var(--df-bg-hover)" }}
            >
              <input
                ref={editLabelRef}
                className="bg-transparent text-[11px] outline-none w-14"
                style={{ color: "var(--df-text)" }}
                value={editLabel}
                placeholder={t("quickCommands.label")}
                onChange={(e) => setEditLabel(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") handleEditSave();
                  if (e.key === "Escape") setEditingId(null);
                }}
              />
              <span className="text-[10px]" style={{ color: "var(--df-text-dimmed)" }}>
                /
              </span>
              <input
                className="bg-transparent text-[11px] font-mono outline-none w-24"
                style={{ color: "var(--df-text-muted)" }}
                value={editCommand}
                placeholder={t("quickCommands.command")}
                onChange={(e) => setEditCommand(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") handleEditSave();
                  if (e.key === "Escape") setEditingId(null);
                }}
              />
              <button
                className="material-icons text-sm text-green-500 hover:text-green-400"
                onClick={handleEditSave}
              >
                check
              </button>
              <button
                className="material-icons text-sm"
                style={{ color: "var(--df-text-dimmed)" }}
                onClick={() => setEditingId(null)}
              >
                close
              </button>
            </div>
          ) : (
            <div
              key={cmd.id}
              className="group flex items-center gap-1 rounded px-2 py-1 cursor-pointer transition-colors shrink-0"
              style={{ backgroundColor: "var(--df-bg-hover)" }}
              title={cmd.command}
              onClick={() => onSend(cmd.command)}
            >
              <span className="text-[11px]" style={{ color: "var(--df-accent)" }}>
                {cmd.label}
              </span>
              <div className="hidden group-hover:flex items-center gap-0.5 ml-0.5">
                <button
                  className="material-icons text-xs transition-colors"
                  style={{ color: "var(--df-text-dimmed)" }}
                  title={t("quickCommands.edit")}
                  onClick={(e) => {
                    e.stopPropagation();
                    startEdit(cmd);
                  }}
                >
                  edit
                </button>
                <button
                  className="material-icons text-xs hover:text-red-400 transition-colors"
                  style={{ color: "var(--df-text-dimmed)" }}
                  title={t("quickCommands.delete")}
                  onClick={(e) => {
                    e.stopPropagation();
                    handleDelete(cmd.id);
                  }}
                >
                  close
                </button>
              </div>
            </div>
          ),
        )}

        {adding ? (
          <div
            className="flex items-center gap-1 rounded px-1.5 py-0.5 shrink-0"
            style={{ backgroundColor: "var(--df-bg-hover)" }}
          >
            <input
              ref={labelInputRef}
              className="bg-transparent text-[11px] outline-none w-14"
              style={{ color: "var(--df-text)" }}
              value={newLabel}
              placeholder={t("quickCommands.label")}
              onChange={(e) => setNewLabel(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") handleAdd();
                if (e.key === "Escape") setAdding(false);
              }}
            />
            <span className="text-[10px]" style={{ color: "var(--df-text-dimmed)" }}>
              /
            </span>
            <input
              className="bg-transparent text-[11px] font-mono outline-none w-24"
              style={{ color: "var(--df-text-muted)" }}
              value={newCommand}
              placeholder={t("quickCommands.command")}
              onChange={(e) => setNewCommand(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") handleAdd();
                if (e.key === "Escape") setAdding(false);
              }}
            />
            <button
              className="material-icons text-sm text-green-500 hover:text-green-400"
              onClick={handleAdd}
            >
              check
            </button>
            <button
              className="material-icons text-sm"
              style={{ color: "var(--df-text-dimmed)" }}
              onClick={() => setAdding(false)}
            >
              close
            </button>
          </div>
        ) : (
          <button
            className="flex items-center gap-0.5 text-[11px] transition-colors shrink-0 px-1.5 py-1 rounded hover:opacity-80"
            style={{ color: "var(--df-text-dimmed)" }}
            onClick={() => setAdding(true)}
          >
            <span className="material-icons text-sm">add</span>
            <span>{t("quickCommands.add")}</span>
          </button>
        )}
      </div>
    </div>
  );
}

export default memo(QuickCommands);
