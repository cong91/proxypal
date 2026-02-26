import type { AppConfig } from "../../lib/tauri";

export interface SettingsBaseProps {
  config: () => AppConfig;
  handleConfigChange: (
    key: keyof AppConfig,
    value: boolean | number | string,
  ) => void;
  saving: () => boolean;
  setConfig: (config: AppConfig) => void;
  setSaving: (saving: boolean) => void;
}
