import { invoke } from "@tauri-apps/api/core";
import { emit } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { FaServer } from "react-icons/fa6";
import { MdAdd, MdExpandMore } from "react-icons/md";
import { SYSTEM_ICONS } from "@/components/icons";
import ChildWindowHeader from "@/components/layout/ChildWindowHeader";
import { LocalTerminal } from "@/components/sessions/LocalTerminal";
import { SerialForm } from "@/components/sessions/SerialForm";
import { SshForm } from "@/components/sessions/SshForm";
import { TelnetForm } from "@/components/sessions/TelnetForm";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Textarea } from "@/components/ui/textarea";
import type { Group, SavedConnection } from "@/types/global";

export default function NewSessionPage() {
  const { t } = useTranslation();
  const params = new URLSearchParams(window.location.search);
  const editId = params.get("edit") ?? undefined;
  const autoConnect = params.get("autoConnect") === "1";

  const [initialData, setInitialData] = useState<SavedConnection | undefined>();
  const [name, setName] = useState("");
  const [groupId, setGroupId] = useState("");
  const [newGroupNamePending, setNewGroupNamePending] = useState("");
  const [description, setDescription] = useState("");
  const [host, setHost] = useState("");
  const [port, setPort] = useState(22);
  const [username, setUsername] = useState("root");
  const [authType, setAuthType] = useState<"password" | "key">("password");
  const [passwordId, setPasswordId] = useState("");
  const [keyId, setKeyId] = useState("");
  const [iconKey, setIconKey] = useState("");
  const [showIconPicker, setShowIconPicker] = useState(false);
  const [connecting, setConnecting] = useState(false);
  const [error, setError] = useState("");
  const [saveSuccess, setSaveSuccess] = useState(false);
  const [groups, setGroups] = useState<Group[]>([]);
  const [showGroupDropdown, setShowGroupDropdown] = useState(false);
  const [newGroupName, setNewGroupName] = useState("");
  const [newGroupParentId, setNewGroupParentId] = useState("");
  const [currentTab, setCurrentTab] = useState("ssh");

  // Serial Settings States
  const [serialPortName, setSerialPortName] = useState("COM1");
  const [baudRate, setBaudRate] = useState("115200");
  const [dataBits, setDataBits] = useState("8");
  const [parity, setParity] = useState("none");
  const [stopBits, setStopBits] = useState("1");

  // Local Terminal States
  const [shellPath, setShellPath] = useState("powershell.exe");
  const [workingDir, setWorkingDir] = useState("");

  const groupRef = useRef<HTMLDivElement>(null);
  const iconPickerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (groupRef.current && !groupRef.current.contains(e.target as Node)) {
        setShowGroupDropdown(false);
        setNewGroupName("");
      }
      if (iconPickerRef.current && !iconPickerRef.current.contains(e.target as Node)) {
        setShowIconPicker(false);
      }
    };
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  useEffect(() => {
    invoke<Group[]>("get_groups")
      .then(setGroups)
      .catch(() => {});

    if (editId) {
      invoke<SavedConnection[]>("get_saved_connections")
        .then((conns) => {
          const found = conns.find((c) => c.id === editId);
          if (found) {
            setInitialData(found);
            setName(found.name);
            setGroupId(found.group_id || "");
            setDescription(found.description || "");
            setHost(found.host);
            setPort(found.port);
            setUsername(found.username);
            setAuthType(found.auth_type as "password" | "key");
            setPasswordId(found.password_id || "");
            setKeyId(found.key_id || "");
            setIconKey(found.icon || "");
          }
        })
        .catch(() => {});
    }
  }, [editId]);

  const resetForm = useCallback(() => {
    setName("");
    setGroupId("");
    setNewGroupNamePending("");
    setDescription("");
    setHost("");
    setPort(22);
    setUsername("root");
    setAuthType("password");
    setPasswordId("");
    setKeyId("");
    setIconKey("");
    setShowIconPicker(false);
    setError("");
    setConnecting(false);
    setSaveSuccess(false);
  }, []);

  const handleClose = () => {
    if (connecting) return;
    getCurrentWindow().close();
  };

  const handleSave = async () => {
    if ((currentTab === "ssh" || currentTab === "telnet") && !host) {
      setError(t("dialog.hostRequired"));
      return;
    }

    setError("");
    setSaveSuccess(false);
    setConnecting(true);

    try {
      let finalGroupId = groupId;
      if (groupId === "new" && newGroupNamePending) {
        finalGroupId = await invoke<string>("save_group", {
          group: {
            id: "",
            name: newGroupNamePending,
            parent_id: newGroupParentId || null,
            sort_order: groups.length,
          },
        });
      }

      const connection: SavedConnection = {
        id: initialData?.id || "",
        name:
          name ||
          (currentTab === "local"
            ? t("dialog.localTerminal")
            : currentTab === "serial"
              ? serialPortName
              : `${host}:${port}`),
        group_id: finalGroupId || undefined,
        description: description || undefined,
        host,
        port,
        username,
        auth_type: authType,
        password_id: authType === "password" && passwordId ? passwordId : undefined,
        key_id: authType === "key" && keyId ? keyId : undefined,
        icon: iconKey || undefined,
      };

      const savedId = await invoke<string>("save_connection", { connection });
      await emit("session-saved");
      if (autoConnect && (initialData?.id || savedId)) {
        await emit("session-connect-after-edit", { connectionId: initialData?.id || savedId });
      }
      resetForm();
      getCurrentWindow().close();
    } catch (e) {
      setError(String(e));
    } finally {
      setConnecting(false);
    }
  };

  return (
    <div className="h-screen flex flex-col overflow-hidden bg-background text-foreground">
      <ChildWindowHeader
        title={t(editId ? "dialog.editConnection" : "dialog.newConnection")}
        onClose={handleClose}
      />

      {/* Body */}
      <Tabs
        value={currentTab}
        onValueChange={setCurrentTab}
        className="flex-1 flex flex-col overflow-hidden"
      >
        <div className="px-5 pt-3 shrink-0">
          <TabsList
            variant="line"
            className="w-full justify-start border-b border-border/40 gap-6 rounded-none h-auto p-0"
          >
            <TabsTrigger value="ssh" className="px-1 py-2 text-xs rounded-none bg-transparent">
              SSH
            </TabsTrigger>
            <TabsTrigger value="local" className="px-1 py-2 text-xs rounded-none bg-transparent">
              {t("dialog.localTerminal")}
            </TabsTrigger>
            <TabsTrigger value="telnet" className="px-1 py-2 text-xs rounded-none bg-transparent">
              Telnet
            </TabsTrigger>
            <TabsTrigger value="serial" className="px-1 py-2 text-xs rounded-none bg-transparent">
              Serial
            </TabsTrigger>
          </TabsList>
        </div>

        <div className="flex-1 overflow-y-auto w-full p-5 space-y-4">
          <div className="flex gap-3 items-end">
            {/* Name + Group */}
            <div className="relative shrink-0" ref={iconPickerRef}>
              <Label className="text-[0.6875rem] text-muted-foreground block mb-1">
                {t("dialog.icon")}
              </Label>
              <Button
                type="button"
                variant="outline"
                className="h-8 w-8 p-0 flex items-center justify-center"
                onClick={() => setShowIconPicker(!showIconPicker)}
                title={iconKey || t("dialog.none")}
              >
                {iconKey && SYSTEM_ICONS[iconKey] ? (
                  (() => {
                    const IconComp = SYSTEM_ICONS[iconKey].icon;
                    return (
                      <IconComp
                        style={{ color: SYSTEM_ICONS[iconKey].color }}
                        className="text-sm"
                      />
                    );
                  })()
                ) : (
                  <FaServer className="text-sm text-muted-foreground" />
                )}
              </Button>
              {showIconPicker && (
                <div className="absolute top-full left-0 mt-1 z-20 border rounded-md shadow-xl bg-popover p-2 min-w-max w-56">
                  <div className="grid grid-cols-7 gap-0.5">
                    <button
                      className={`w-7 h-7 flex items-center justify-center rounded transition-colors hover:bg-accent ${!iconKey ? "bg-primary/15 ring-1 ring-primary/40" : ""}`}
                      title={t("dialog.none")}
                      onClick={() => {
                        setIconKey("");
                        setShowIconPicker(false);
                      }}
                    >
                      <FaServer className="text-sm text-muted-foreground" />
                    </button>
                    {Object.entries(SYSTEM_ICONS).map(([key, def]) => {
                      const IconComp = def.icon;
                      return (
                        <button
                          key={key}
                          className={`w-7 h-7 flex items-center justify-center rounded transition-colors hover:bg-accent ${iconKey === key ? "bg-primary/15 ring-1 ring-primary/40" : ""}`}
                          title={key}
                          onClick={() => {
                            setIconKey(key);
                            setShowIconPicker(false);
                          }}
                        >
                          <IconComp style={{ color: def.color }} className="text-sm" />
                        </button>
                      );
                    })}
                  </div>
                </div>
              )}
            </div>
            <div className="flex-1">
              <Label className="text-[0.6875rem] text-muted-foreground">
                {t("dialog.connectionName")}
              </Label>
              <Input
                className="mt-1 text-xs h-8"
                placeholder={t("dialog.serverPlaceholder")}
                value={name}
                onChange={(e) => setName(e.target.value)}
              />
            </div>
            <div className="w-44 relative" ref={groupRef}>
              <Label className="text-[0.6875rem] text-muted-foreground">{t("dialog.group")}</Label>
              <Button
                type="button"
                variant="outline"
                className="w-full mt-1 h-8 justify-between text-xs font-normal"
                onClick={() => setShowGroupDropdown(!showGroupDropdown)}
              >
                <span className={`truncate ${groupId ? "" : "text-muted-foreground"}`}>
                  {groupId === "new"
                    ? newGroupNamePending
                    : groupId
                      ? (() => {
                          const parts: string[] = [];
                          let cur: string | undefined = groupId;
                          while (cur) {
                            const g = groups.find((g) => g.id === cur);
                            if (!g) break;
                            parts.unshift(g.name);
                            cur = g.parent_id;
                          }
                          return parts.join(" / ");
                        })()
                      : t("dialog.none")}
                </span>
                <MdExpandMore className="text-xs text-muted-foreground shrink-0" />
              </Button>
              {showGroupDropdown && (
                <div className="absolute top-full left-0 right-0 mt-1 border rounded-md shadow-xl z-10 overflow-hidden bg-popover max-h-60 overflow-y-auto">
                  <div
                    className={`px-3 py-1.5 text-xs cursor-pointer transition-colors hover:bg-accent ${!groupId ? "bg-primary/15 text-primary" : "text-muted-foreground"}`}
                    onClick={() => {
                      setGroupId("");
                      setNewGroupNamePending("");
                      setNewGroupParentId("");
                      setShowGroupDropdown(false);
                    }}
                  >
                    {t("dialog.none")}
                  </div>
                  {(() => {
                    const getDepth = (g: Group): number => {
                      let d = 0;
                      let cur: string | undefined = g.parent_id;
                      while (cur) {
                        d++;
                        const parent = groups.find((x) => x.id === cur);
                        cur = parent?.parent_id;
                      }
                      return d;
                    };
                    const sorted = [...groups].sort((a, b) => a.sort_order - b.sort_order);
                    const buildTree = (parentId: string | undefined): Group[] => {
                      const children = sorted.filter(
                        (g) => (g.parent_id || undefined) === parentId,
                      );
                      return children.flatMap((g) => [g, ...buildTree(g.id)]);
                    };
                    const ordered = buildTree(undefined);
                    return ordered.map((g) => {
                      const depth = getDepth(g);
                      return (
                        <div
                          key={g.id}
                          className={`py-1.5 text-xs cursor-pointer transition-colors hover:bg-accent ${groupId === g.id ? "bg-primary/15 text-primary" : ""}`}
                          style={{ paddingLeft: `${12 + depth * 16}px`, paddingRight: "12px" }}
                          onClick={() => {
                            setGroupId(g.id);
                            setNewGroupNamePending("");
                            setNewGroupParentId("");
                            setShowGroupDropdown(false);
                          }}
                        >
                          {g.name}
                        </div>
                      );
                    });
                  })()}
                  <div className="p-1.5 border-t">
                    <div className="flex items-center gap-1.5">
                      <Input
                        className="flex-1 min-w-0 h-7 text-xs"
                        placeholder={t("dialog.newGroupPlaceholder")}
                        value={newGroupName}
                        onChange={(e) => setNewGroupName(e.target.value)}
                        onKeyDown={(e) => {
                          if (e.key === "Enter" && newGroupName.trim()) {
                            setGroupId("new");
                            setNewGroupNamePending(newGroupName.trim());
                            setNewGroupParentId(groupId && groupId !== "new" ? groupId : "");
                            setNewGroupName("");
                            setShowGroupDropdown(false);
                          }
                        }}
                      />
                      <Button
                        variant="ghost"
                        size="icon-xs"
                        disabled={!newGroupName.trim()}
                        onClick={() => {
                          if (newGroupName.trim()) {
                            setGroupId("new");
                            setNewGroupNamePending(newGroupName.trim());
                            setNewGroupParentId(groupId && groupId !== "new" ? groupId : "");
                            setNewGroupName("");
                            setShowGroupDropdown(false);
                          }
                        }}
                      >
                        <MdAdd className="text-sm" />
                      </Button>
                    </div>
                  </div>
                </div>
              )}
            </div>
          </div>

          <TabsContent value="ssh" className="space-y-4 m-0 border-0 outline-none w-full">
            <SshForm
              host={host}
              setHost={setHost}
              port={port}
              setPort={setPort}
              username={username}
              setUsername={setUsername}
              authType={authType}
              setAuthType={setAuthType as any}
              passwordId={passwordId}
              setPasswordId={setPasswordId}
              keyId={keyId}
              setKeyId={setKeyId}
            />
          </TabsContent>

          <TabsContent value="local" className="space-y-4 m-0 border-0 outline-none w-full">
            <LocalTerminal
              shellPath={shellPath}
              setShellPath={setShellPath}
              workingDir={workingDir}
              setWorkingDir={setWorkingDir}
            />
          </TabsContent>

          <TabsContent value="telnet" className="space-y-4 m-0 border-0 outline-none w-full">
            <TelnetForm host={host} setHost={setHost} port={port} setPort={setPort} />
          </TabsContent>

          <TabsContent value="serial" className="space-y-4 m-0 border-0 outline-none w-full">
            <SerialForm
              serialPortName={serialPortName}
              setSerialPortName={setSerialPortName}
              baudRate={baudRate}
              setBaudRate={setBaudRate}
              dataBits={dataBits}
              setDataBits={setDataBits}
              parity={parity}
              setParity={setParity}
              stopBits={stopBits}
              setStopBits={setStopBits}
            />
          </TabsContent>

          <div className="space-y-4 mt-8">
            {/* Description */}
            <div>
              <Label className="text-[0.6875rem] text-muted-foreground">
                {t("dialog.description")}
              </Label>
              <Textarea
                rows={2}
                placeholder={t("dialog.descriptionPlaceholder")}
                className="mt-1 text-xs resize-none"
                value={description}
                onChange={(e) => setDescription(e.target.value)}
              />
            </div>

            {/* Messages */}
            {error && (
              <div className="p-2 bg-destructive/10 border border-destructive/30 rounded text-xs text-red-400">
                {error}
              </div>
            )}
            {saveSuccess && (
              <div className="p-2 bg-green-500/10 border border-green-500/30 rounded text-xs text-green-400">
                {t("dialog.connectionSaved")}
              </div>
            )}
          </div>
        </div>
      </Tabs>

      {/* Footer */}
      <div className="px-5 py-3 border-t flex justify-end gap-2 shrink-0">
        <Button variant="ghost" size="sm" className="text-xs" onClick={handleClose}>
          {t("dialog.cancel")}
        </Button>
        <Button
          size="sm"
          className="text-xs"
          onClick={handleSave}
          disabled={connecting || ((currentTab === "ssh" || currentTab === "telnet") && !host)}
        >
          {connecting ? t("dialog.saving") : t("dialog.save")}
        </Button>
      </div>
    </div>
  );
}
