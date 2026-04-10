import { useTranslation } from "react-i18next";
import { SelectItem } from "@/components/ui/select";
import { useApp } from "@/context/AppContext";
import { NumberInput } from "../ui/number-input";
import { SettingInput, SettingRow, SettingSelect, SettingSwitch } from "./SettingFormItems";

export function SecurityTab() {
  const { t } = useTranslation();
  const { appSettings, updateAppSettings } = useApp();

  return (
    <div className="space-y-6">
      <div className="space-y-4">
        <h4 className="font-semibold text-sm">{t("settings.sessionSecurity")}</h4>

        <SettingRow
          label={t("settings.enableScreenLock")}
          desc={t("settings.enableScreenLockDesc")}
        >
          <SettingSwitch
            checked={appSettings.security.enable_screen_lock}
            onChange={(v) =>
              updateAppSettings({
                security: { ...appSettings.security, enable_screen_lock: v },
              })
            }
          />
        </SettingRow>

        {appSettings.security.enable_screen_lock && (
          <>
            <SettingRow
              label={t("settings.idleLockMinutes")}
              desc={t("settings.idleLockMinutesDesc")}
            >
              <div className="flex w-full items-center gap-3 sm:w-auto">
                <NumberInput
                  min={0}
                  max={1440}
                  className="w-full sm:w-28"
                  value={appSettings.security.idle_lock_minutes}
                  onChange={(v) =>
                    updateAppSettings({
                      security: { ...appSettings.security, idle_lock_minutes: v || 0 },
                    })
                  }
                />
                <span className="shrink-0 text-sm text-muted-foreground">
                  {t("common.minutes")}
                </span>
              </div>
            </SettingRow>

            <SettingInput
              label={t("settings.lockPassword")}
              desc={t("settings.lockPasswordDesc")}
              type="password"
              placeholder={
                appSettings.security.lock_password === "__SET__"
                  ? "••••••••"
                  : t("settings.lockPasswordPlaceholder")
              }
              value={
                appSettings.security.lock_password === "__SET__"
                  ? ""
                  : appSettings.security.lock_password || ""
              }
              onChange={(e) => {
                const val = e.target.value;
                // Empty string clears the password; non-empty sets new plaintext
                updateAppSettings({
                  security: { ...appSettings.security, lock_password: val || undefined },
                });
              }}
            />
          </>
        )}

        <SettingSelect
          label={t("settings.hostKeyPolicy")}
          desc={t("settings.hostKeyPolicyDesc")}
          value={appSettings.security.host_key_policy}
          onValueChange={(v) =>
            updateAppSettings({ security: { ...appSettings.security, host_key_policy: v } })
          }
        >
          <SelectItem value="strict">{t("settings.hostKeyStrict")}</SelectItem>
          <SelectItem value="prompt">{t("settings.hostKeyPrompt")}</SelectItem>
          <SelectItem value="accept">{t("settings.hostKeyAccept")}</SelectItem>
        </SettingSelect>
      </div>
    </div>
  );
}
