import { invoke } from "@tauri-apps/api/core";
import { memo } from "react";
import { useTranslation } from "react-i18next";
import type { Tab } from "../../types";

interface TabBarProps {
  tabs: Tab[];
  activeTabId: string | null;
  onTabChange: (tabId: string) => void;
  onTabClose: (tabId: string) => void;
  onAddTab: () => void;
}

/** Tab strip for terminal sessions. Closes backend session on tab close. */
function TabBar({
  tabs,
  activeTabId,
  onTabChange,
  onTabClose,
  onAddTab,
}: TabBarProps) {
  const { t } = useTranslation();

  const handleClose = (e: React.MouseEvent, tab: Tab) => {
    e.stopPropagation();
    // Close the backend session
    invoke("close_session", { sessionId: tab.sessionId }).catch(() => {});
    onTabClose(tab.id);
  };

  return (
    <div
      className="flex h-9 overflow-x-auto terminal-scroll shrink-0 border-b"
      style={{ backgroundColor: "var(--df-bg-panel)", borderColor: "var(--df-border)" }}
    >
      {tabs.map((tab) => (
        <div
          key={tab.id}
          className={`group flex items-center px-4 gap-2 border-r text-xs font-medium cursor-pointer transition-colors ${
            activeTabId === tab.id ? "active-tab" : ""
          } ${activeTabId !== tab.id ? "df-hover" : ""}`}
          style={{
            borderColor: "var(--df-border)",
            color: activeTabId === tab.id ? "var(--df-text)" : "var(--df-text-muted)",
          }}
          onClick={() => onTabChange(tab.id)}
        >
          <span className="material-icons text-sm">{tab.type === "SSH" ? "dns" : "terminal"}</span>
          <span className="whitespace-nowrap max-w-[160px] truncate">{tab.name}</span>
          <span
            className="material-icons text-[10px] hover:text-red-500 transition-colors"
            style={{ color: "var(--df-text-dimmed)" }}
            onClick={(e) => handleClose(e, tab)}
          >
            close
          </span>
        </div>
      ))}
      <button
        className="px-3 transition-colors df-hover"
        style={{ color: "var(--df-text-muted)" }}
        onClick={onAddTab}
        title={t("terminal.newConnection")}
      >
        <span className="material-icons text-base">add</span>
      </button>
    </div>
  );
}

export default memo(TabBar);
