import React, { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { SettingContainer, SettingsGroup, ToggleSwitch } from "../ui";
import { Input } from "../ui/Input";
import { useSettings } from "../../hooks/useSettings";

const UrlField: React.FC<{
  value: string;
  disabled: boolean;
  onCommit: (value: string) => void;
}> = ({ value, disabled, onCommit }) => {
  const { t } = useTranslation();
  const [localValue, setLocalValue] = useState(value);

  useEffect(() => {
    setLocalValue(value);
  }, [value]);

  return (
    <SettingContainer
      title={t("settings.general.cleanup.baseUrl.label")}
      description={t("settings.general.cleanup.baseUrl.description")}
      descriptionMode="tooltip"
      layout="horizontal"
      grouped
      disabled={disabled}
    >
      <Input
        type="text"
        value={localValue}
        onChange={(event) => setLocalValue(event.target.value)}
        onBlur={() => onCommit(localValue.trim())}
        placeholder={t("settings.general.cleanup.baseUrl.placeholder")}
        variant="compact"
        disabled={disabled}
        className="min-w-[280px]"
      />
    </SettingContainer>
  );
};

const ModelField: React.FC<{
  value: string;
  disabled: boolean;
  onCommit: (value: string) => void;
}> = ({ value, disabled, onCommit }) => {
  const { t } = useTranslation();
  const [localValue, setLocalValue] = useState(value);

  useEffect(() => {
    setLocalValue(value);
  }, [value]);

  return (
    <SettingContainer
      title={t("settings.general.cleanup.model.label")}
      description={t("settings.general.cleanup.model.description")}
      descriptionMode="tooltip"
      layout="horizontal"
      grouped
      disabled={disabled}
    >
      <Input
        type="text"
        value={localValue}
        onChange={(event) => setLocalValue(event.target.value)}
        onBlur={() => onCommit(localValue.trim())}
        placeholder={t("settings.general.cleanup.model.placeholder")}
        variant="compact"
        disabled={disabled}
        className="min-w-[180px]"
      />
    </SettingContainer>
  );
};

export const LocalCleanupSettings: React.FC = () => {
  const { t } = useTranslation();
  const { getSetting, updateSetting, isUpdating } = useSettings();
  const enabled = getSetting("cleanup_enabled") ?? false;
  const baseUrl = getSetting("cleanup_base_url") ?? "http://127.0.0.1:11434/v1";
  const model = getSetting("cleanup_model") ?? "";
  const busy =
    isUpdating("cleanup_enabled") ||
    isUpdating("cleanup_base_url") ||
    isUpdating("cleanup_model");

  return (
    <SettingsGroup title={t("settings.general.cleanup.title")}>
      <ToggleSwitch
        checked={enabled}
        onChange={(value) => updateSetting("cleanup_enabled", value)}
        isUpdating={isUpdating("cleanup_enabled")}
        label={t("settings.general.cleanup.enabled.label")}
        description={t("settings.general.cleanup.enabled.description")}
        descriptionMode="tooltip"
        grouped
      />
      <UrlField
        value={baseUrl}
        disabled={busy}
        onCommit={(value) => updateSetting("cleanup_base_url", value)}
      />
      <ModelField
        value={model}
        disabled={busy}
        onCommit={(value) => updateSetting("cleanup_model", value)}
      />
    </SettingsGroup>
  );
};
