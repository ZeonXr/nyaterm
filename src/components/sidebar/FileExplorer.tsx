import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

interface FileEntry {
  name: string;
  is_dir: boolean;
  size: number;
  permissions: string;
}

interface FileExplorerProps {
  activeSessionId: string | null;
}

function getFileIcon(entry: FileEntry): { icon: string; color: string } {
  if (entry.is_dir) return { icon: "folder", color: "#eab308" }; // yellow-500
  const ext = entry.name.split(".").pop()?.toLowerCase() ?? "";
  switch (ext) {
    case "js":
    case "ts":
    case "jsx":
    case "tsx":
      return { icon: "javascript", color: "#60a5fa" };
    case "py":
      return { icon: "code", color: "#4ade80" };
    case "sh":
    case "bash":
      return { icon: "terminal", color: "#22d3ee" };
    case "json":
    case "yaml":
    case "yml":
    case "toml":
      return { icon: "description", color: "#fb923c" };
    case "md":
    case "txt":
      return { icon: "article", color: "var(--df-text-muted)" };
    default:
      return { icon: "insert_drive_file", color: "var(--df-text-muted)" };
  }
}

function formatSize(bytes: number): string {
  if (bytes === 0) return "-";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

/** Remote file browser for active SSH session. Lists dirs/files, supports navigation. */
export default function FileExplorer({ activeSessionId }: FileExplorerProps) {
  const { t } = useTranslation();
  const [files, setFiles] = useState<FileEntry[]>([]);
  const [currentPath, setCurrentPath] = useState("");
  const [homeDir, setHomeDir] = useState("");
  const [selectedFile, setSelectedFile] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadDirectory = useCallback(
    async (path: string) => {
      if (!activeSessionId) return;
      setLoading(true);
      setError(null);

      try {
        const entries = await invoke<FileEntry[]>("list_remote_dir", {
          sessionId: activeSessionId,
          path,
        });
        entries.sort((a, b) => {
          if (a.is_dir !== b.is_dir) return a.is_dir ? -1 : 1;
          return a.name.localeCompare(b.name);
        });
        setFiles(entries);
        setCurrentPath(path);
      } catch (e) {
        setError(String(e));
        setFiles([]);
      } finally {
        setLoading(false);
      }
    },
    [activeSessionId],
  );

  useEffect(() => {
    let cancelled = false;
    if (activeSessionId) {
      (async () => {
        try {
          const home = await invoke<string>("get_home_dir", { sessionId: activeSessionId });
          if (cancelled) return;
          setHomeDir(home);
          loadDirectory(home);
        } catch {
          if (cancelled) return;
          loadDirectory("~");
        }
      })();
    } else {
      setFiles([]);
      setCurrentPath("");
      setHomeDir("");
    }
    return () => {
      cancelled = true;
    };
  }, [activeSessionId, loadDirectory]);

  const handleItemClick = (entry: FileEntry) => {
    if (entry.is_dir) {
      loadDirectory(`${currentPath}/${entry.name}`);
    } else {
      setSelectedFile(entry.name);
    }
  };

  const handleGoUp = () => {
    if (!currentPath || currentPath === "/") return;
    const parts = currentPath.split("/");
    parts.pop();
    loadDirectory(parts.join("/") || "/");
  };

  const displayPath = (() => {
    if (!homeDir || !currentPath) return currentPath || "~";
    if (currentPath === homeDir) return "~";
    if (currentPath.startsWith(`${homeDir}/`)) return `~${currentPath.slice(homeDir.length)}`;
    return currentPath;
  })();

  return (
    <aside
      className="h-full flex flex-col overflow-hidden"
      style={{ backgroundColor: "var(--df-bg-panel)" }}
    >
      <div
        className="p-2 text-[10px] uppercase tracking-wider font-bold border-b flex justify-between items-center"
        style={{ color: "var(--df-text-muted)", borderColor: "var(--df-border)" }}
      >
        <span>{t("panel.fileExplorer")}</span>
        <div className="flex gap-1">
          {activeSessionId && (
            <>
              <span
                className="material-icons text-xs cursor-pointer hover:opacity-80 transition-opacity"
                style={{ color: "var(--df-text-muted)" }}
                onClick={handleGoUp}
                title={t("fileExplorer.goUp")}
              >
                arrow_upward
              </span>
              <span
                className="material-icons text-xs cursor-pointer hover:opacity-80 transition-opacity"
                style={{ color: "var(--df-text-muted)" }}
                onClick={() => loadDirectory(currentPath)}
                title={t("fileExplorer.refresh")}
              >
                refresh
              </span>
            </>
          )}
        </div>
      </div>

      {activeSessionId && (
        <div
          className="px-2 py-1 text-[10px] border-b font-mono truncate"
          style={{ color: "var(--df-text-dimmed)", borderColor: "var(--df-border)" }}
        >
          {displayPath}
        </div>
      )}

      <div className="flex-1 overflow-y-auto p-2 text-sm terminal-scroll">
        {!activeSessionId ? (
          <div className="text-center py-8 text-xs" style={{ color: "var(--df-text-dimmed)" }}>
            <div className="material-icons text-xl block mb-2">folder_off</div>
            <div className="text-sm block mb-2">{t("fileExplorer.connectToSession")}</div>
          </div>
        ) : loading ? (
          <div className="text-center py-4 text-xs" style={{ color: "var(--df-text-dimmed)" }}>
            {t("fileExplorer.loading")}
          </div>
        ) : error ? (
          <div className="text-center text-red-400 py-4 text-xs">{error}</div>
        ) : files.length === 0 ? (
          <div className="text-center py-4 text-xs" style={{ color: "var(--df-text-dimmed)" }}>
            {t("fileExplorer.emptyDirectory")}
          </div>
        ) : (
          <ul className="space-y-0.5">
            {files.map((entry) => {
              const { icon, color } = getFileIcon(entry);
              const isSelected = selectedFile === entry.name;
              return (
                <li
                  key={entry.name}
                  className="flex items-center gap-2 px-2 py-1 rounded cursor-pointer transition-colors"
                  style={{
                    backgroundColor: isSelected
                      ? "color-mix(in srgb, var(--df-primary) 10%, transparent)"
                      : undefined,
                    color: isSelected ? "var(--df-primary)" : "var(--df-text)",
                  }}
                  onMouseEnter={(e) => {
                    if (!isSelected) e.currentTarget.style.backgroundColor = "var(--df-bg-hover)";
                  }}
                  onMouseLeave={(e) => {
                    if (!isSelected) e.currentTarget.style.backgroundColor = "";
                  }}
                  onClick={() => handleItemClick(entry)}
                  title={`${entry.permissions} ${formatSize(entry.size)}`}
                >
                  <span
                    className="material-icons text-base"
                    style={{ color: isSelected ? "var(--df-primary)" : color }}
                  >
                    {icon}
                  </span>
                  <span className="flex-1 truncate text-xs">{entry.name}</span>
                  {!entry.is_dir && (
                    <span className="text-[10px]" style={{ color: "var(--df-text-dimmed)" }}>
                      {formatSize(entry.size)}
                    </span>
                  )}
                </li>
              );
            })}
          </ul>
        )}
      </div>
    </aside>
  );
}
