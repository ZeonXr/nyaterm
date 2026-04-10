import { useTranslation } from "react-i18next";
import { MdCheck, MdClose } from "react-icons/md";
import { Label } from "@/components/ui/label";
import { SelectItem } from "@/components/ui/select";
import { useApp } from "@/context/AppContext";
import { SettingInput, SettingSelect } from "./SettingFormItems";

const TARGET_LANGUAGES = [
  { value: "zh-CN", label: "中文 (简体)" },
  { value: "zh-TW", label: "中文 (繁體)" },
  { value: "en", label: "English" },
  { value: "ja", label: "日本語" },
  { value: "ko", label: "한국어" },
  { value: "fr", label: "Français" },
  { value: "de", label: "Deutsch" },
  { value: "es", label: "Español" },
  { value: "pt", label: "Português" },
  { value: "ru", label: "Русский" },
  { value: "it", label: "Italiano" },
  { value: "ar", label: "العربية" },
  { value: "th", label: "ไทย" },
  { value: "vi", label: "Tiếng Việt" },
];

function ProviderStatus({ configured, free }: { configured: boolean; free?: boolean }) {
  const { t } = useTranslation();
  if (free) {
    return (
      <span className="inline-flex items-center gap-1 text-xs text-muted-foreground">
        <MdCheck className="text-green-500" />
        {t("settings.noKeyRequired")}
      </span>
    );
  }
  return configured ? (
    <span className="inline-flex items-center gap-1 text-xs text-green-600">
      <MdCheck />
      {t("settings.configured")}
    </span>
  ) : (
    <span className="inline-flex items-center gap-1 text-xs text-muted-foreground">
      <MdClose />
      {t("settings.notConfigured")}
    </span>
  );
}

export function TranslationTab() {
  const { t } = useTranslation();
  const { appSettings, updateAppSettings } = useApp();
  const ts = appSettings.translation;

  const update = (patch: Partial<typeof ts>) =>
    updateAppSettings({ translation: { ...ts, ...patch } });

  return (
    <div className="space-y-6">
      <SettingSelect
        label={t("settings.targetLanguage")}
        desc={t("settings.targetLanguageDesc")}
        value={ts.target_language || "zh-CN"}
        onValueChange={(v) => update({ target_language: v })}
      >
        {TARGET_LANGUAGES.map((lang) => (
          <SelectItem key={lang.value} value={lang.value}>
            {lang.label}
          </SelectItem>
        ))}
      </SettingSelect>

      <div>
        <Label className="font-medium text-sm">{t("settings.translationProviders")}</Label>
        <p className="text-xs text-muted-foreground mb-3">
          {t("settings.translationProvidersDesc")}
        </p>

        <div className="space-y-4">
          {/* Free providers */}
          <div className="rounded-md border p-3 space-y-1">
            <div className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
              <span className="text-sm font-medium">{t("translation.google")}</span>
              <ProviderStatus configured free />
            </div>
          </div>

          <div className="rounded-md border p-3 space-y-1">
            <div className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
              <span className="text-sm font-medium">{t("translation.microsoft")}</span>
              <ProviderStatus configured free />
            </div>
          </div>

          {/* DeepL */}
          <div className="rounded-md border p-3 space-y-3">
            <div className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
              <span className="text-sm font-medium">{t("translation.deepl")}</span>
              <ProviderStatus configured={!!ts.deepl_api_key} />
            </div>
            <SettingInput
              label={t("settings.apiKey")}
              type="password"
              placeholder="xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx:fx"
              value={ts.deepl_api_key}
              onChange={(e) => update({ deepl_api_key: e.target.value })}
            />
          </div>

          {/* Baidu */}
          <div className="rounded-md border p-3 space-y-3">
            <div className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
              <span className="text-sm font-medium">{t("translation.baidu")}</span>
              <ProviderStatus configured={!!(ts.baidu_app_id && ts.baidu_app_key)} />
            </div>
            <div className="grid gap-3 sm:grid-cols-2">
              <SettingInput
                label={t("settings.appId")}
                placeholder="App ID"
                value={ts.baidu_app_id}
                onChange={(e) => update({ baidu_app_id: e.target.value })}
              />
              <SettingInput
                label={t("settings.appKey")}
                type="password"
                placeholder="App Key"
                value={ts.baidu_app_key}
                onChange={(e) => update({ baidu_app_key: e.target.value })}
              />
            </div>
          </div>

          {/* Ali */}
          <div className="rounded-md border p-3 space-y-3">
            <div className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
              <span className="text-sm font-medium">{t("translation.ali")}</span>
              <ProviderStatus configured={!!(ts.ali_app_id && ts.ali_app_key)} />
            </div>
            <div className="grid gap-3 sm:grid-cols-2">
              <SettingInput
                label={t("settings.appId")}
                placeholder="Access Key ID"
                value={ts.ali_app_id}
                onChange={(e) => update({ ali_app_id: e.target.value })}
              />
              <SettingInput
                label={t("settings.appKey")}
                type="password"
                placeholder="Access Key Secret"
                value={ts.ali_app_key}
                onChange={(e) => update({ ali_app_key: e.target.value })}
              />
            </div>
          </div>

          {/* Youdao */}
          <div className="rounded-md border p-3 space-y-3">
            <div className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
              <span className="text-sm font-medium">{t("translation.youdao")}</span>
              <ProviderStatus configured={!!(ts.youdao_app_id && ts.youdao_app_key)} />
            </div>
            <div className="grid gap-3 sm:grid-cols-2">
              <SettingInput
                label={t("settings.appId")}
                placeholder="App ID"
                value={ts.youdao_app_id}
                onChange={(e) => update({ youdao_app_id: e.target.value })}
              />
              <SettingInput
                label={t("settings.appKey")}
                type="password"
                placeholder="App Key"
                value={ts.youdao_app_key}
                onChange={(e) => update({ youdao_app_key: e.target.value })}
              />
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
