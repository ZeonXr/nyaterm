import { useTranslation } from "react-i18next";
import { SelectItem } from "@/components/ui/select";
import { useApp } from "@/context/AppContext";
import {
  SettingInput,
  SettingNumberInput,
  SettingRow,
  SettingSelect,
  SettingSwitch,
} from "./SettingFormItems";

export function ProxyTab() {
  const { t } = useTranslation();
  const { appSettings, updateAppSettings } = useApp();

  return (
    <div className="space-y-4">
      <SettingRow label={t("settings.enableProxy")} desc={t("settings.enableProxyDesc")}>
        <SettingSwitch
          checked={appSettings.proxy.enabled}
          onChange={(v) => updateAppSettings({ proxy: { ...appSettings.proxy, enabled: v } })}
        />
      </SettingRow>

      <div
        className={`space-y-4 ${!appSettings.proxy.enabled ? "opacity-50 pointer-events-none" : ""}`}
      >
        <SettingSelect
          label={t("settings.proxyProtocol")}
          value={appSettings.proxy.protocol}
          onValueChange={(v) => updateAppSettings({ proxy: { ...appSettings.proxy, protocol: v } })}
        >
          <SelectItem value="socks5">SOCKS5</SelectItem>
          <SelectItem value="http">HTTP</SelectItem>
        </SettingSelect>

        <div className="flex flex-col gap-2 sm:flex-row">
          <div className="min-w-0 flex-1">
            <SettingInput
              label={t("settings.proxyHost")}
              placeholder="127.0.0.1"
              value={appSettings.proxy.host}
              onChange={(e) =>
                updateAppSettings({ proxy: { ...appSettings.proxy, host: e.target.value } })
              }
            />
          </div>
          <div className="w-full sm:w-32">
            <SettingNumberInput
              label={t("settings.proxyPort")}
              min={1}
              max={65535}
              value={appSettings.proxy.port || 0}
              onChange={(v) => updateAppSettings({ proxy: { ...appSettings.proxy, port: v || 0 } })}
            />
          </div>
        </div>
      </div>
    </div>
  );
}
