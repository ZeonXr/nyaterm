import { downloadDir } from "@tauri-apps/api/path";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { MdFolderOpen } from "react-icons/md";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { SelectItem } from "@/components/ui/select";
import { useApp } from "@/context/AppContext";
import {
  SettingInput,
  SettingNumberInput,
  SettingRow,
  SettingSelect,
  SettingSwitch,
} from "./SettingFormItems";

function PathPickerInput({
  label,
  desc,
  value,
  placeholder,
  onChange,
  directory = true,
  filters,
}: {
  label: string;
  desc?: string;
  value: string;
  placeholder?: string;
  onChange: (v: string) => void;
  directory?: boolean;
  filters?: { name: string; extensions: string[] }[];
}) {
  const { t } = useTranslation();

  const handleBrowse = async () => {
    const selected = await openDialog({
      directory,
      filters: directory ? undefined : filters,
    });
    if (selected && typeof selected === "string") {
      onChange(selected);
    }
  };

  return (
    <div className="space-y-1">
      <Label className="font-medium text-sm">{label}</Label>
      {desc && <p className="text-xs text-muted-foreground">{desc}</p>}
      <div className="flex gap-2">
        <Input
          className="text-sm flex-1"
          placeholder={placeholder}
          value={value}
          onChange={(e) => onChange(e.target.value)}
        />
        <Button
          variant="outline"
          size="sm"
          className="shrink-0 gap-1.5"
          onClick={handleBrowse}
        >
          <MdFolderOpen className="text-sm" />
          {t("settings.browse")}
        </Button>
      </div>
    </div>
  );
}

export function TransferTab() {
  const { t } = useTranslation();
  const { appSettings, updateAppSettings } = useApp();
  const transfer = appSettings.transfer;
  const [defaultDownloadDir, setDefaultDownloadDir] = useState("");

  useEffect(() => {
    downloadDir().then(setDefaultDownloadDir).catch(() => {});
  }, []);

  const update = (patch: Partial<typeof transfer>) =>
    updateAppSettings({ transfer: { ...transfer, ...patch } });

  return (
    <div className="space-y-6">
      <PathPickerInput
        label={t("settings.downloadPath")}
        desc={t("settings.downloadPathDesc")}
        value={transfer.download_path}
        placeholder={defaultDownloadDir}
        onChange={(v) => update({ download_path: v })}
      />

      <SettingRow
        label={t("settings.askSaveLocation")}
        desc={t("settings.askSaveLocationDesc")}
      >
        <SettingSwitch
          checked={transfer.ask_save_location}
          onChange={(v) => update({ ask_save_location: v })}
        />
      </SettingRow>

      <PathPickerInput
        label={t("settings.defaultEditor")}
        desc={t("settings.defaultEditorDesc")}
        value={transfer.default_editor}
        placeholder={t("settings.defaultEditorDesc")}
        onChange={(v) => update({ default_editor: v })}
        directory={false}
        filters={[{ name: "Executable", extensions: ["exe", "cmd", "bat", "com", "app", "sh", ""] }]}
      />

      <PathPickerInput
        label={t("settings.recordingPath")}
        desc={t("settings.recordingPathDesc")}
        value={transfer.recording_path}
        placeholder={defaultDownloadDir}
        onChange={(v) => update({ recording_path: v })}
      />

      <div className="space-y-4">
        <SettingNumberInput
          label={t("settings.downloadThreads")}
          desc={t("settings.downloadThreadsDesc")}
          min={1}
          max={10}
          value={transfer.download_threads}
          onChange={(v) => update({ download_threads: v })}
        />

        <SettingNumberInput
          label={t("settings.uploadThreads")}
          desc={t("settings.uploadThreadsDesc")}
          min={1}
          max={10}
          value={transfer.upload_threads}
          onChange={(v) => update({ upload_threads: v })}
        />
      </div>

      <SettingSelect
        label={t("settings.duplicateStrategy")}
        desc={t("settings.duplicateStrategyDesc")}
        value={transfer.duplicate_strategy}
        onValueChange={(v) => update({ duplicate_strategy: v })}
      >
        <SelectItem value="overwrite">{t("settings.strategyOverwrite")}</SelectItem>
        <SelectItem value="skip">{t("settings.strategySkip")}</SelectItem>
        <SelectItem value="rename">{t("settings.strategyRename")}</SelectItem>
        <SelectItem value="ask">{t("settings.strategyAsk")}</SelectItem>
      </SettingSelect>

      <SettingRow
        label={t("settings.preserveTimestamps")}
        desc={t("settings.preserveTimestampsDesc")}
      >
        <SettingSwitch
          checked={transfer.preserve_timestamps}
          onChange={(v) => update({ preserve_timestamps: v })}
        />
      </SettingRow>

      <SettingRow
        label={t("settings.resumeBrokenTransfer")}
        desc={t("settings.resumeBrokenTransferDesc")}
      >
        <SettingSwitch
          checked={transfer.resume_broken_transfer}
          onChange={(v) => update({ resume_broken_transfer: v })}
        />
      </SettingRow>

      <SettingInput
        label={t("settings.defaultFilePermissions")}
        desc={t("settings.defaultFilePermissionsDesc")}
        placeholder="644"
        value={transfer.default_file_permissions}
        onChange={(e) => update({ default_file_permissions: e.target.value })}
      />

      <SettingNumberInput
        label={t("settings.maxTransferRetries")}
        desc={t("settings.maxTransferRetriesDesc")}
        min={0}
        max={10}
        value={transfer.max_transfer_retries}
        onChange={(v) => update({ max_transfer_retries: v })}
      />

      <SettingNumberInput
        label={t("settings.transferBufferSize")}
        desc={t("settings.transferBufferSizeDesc")}
        min={8}
        max={256}
        step={8}
        value={transfer.transfer_buffer_size}
        onChange={(v) => update({ transfer_buffer_size: v })}
      />
    </div>
  );
}
