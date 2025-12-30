import { invoke } from "@tauri-apps/api/core";
import { appDataDir, homeDir } from "@tauri-apps/api/path";
import { toast } from "svelte-sonner";

export interface AppSettings {
  replayDownloadPath: string;
  mapDownloadPath: string;
  hideShortReplays: boolean;
  maxApiRequestsTps: number;
}

export class SettingsStore {
  private constructor(private _settings: AppSettings) {}

  static create = async () => {
    const settings = await SettingsStore.loadSettings();
    const store = new SettingsStore(settings);
    return store;
  };

  private rateLimitListeners: ((tps: number) => void)[] = [];

  onRateLimitChange(callback: (tps: number) => void) {
    this.rateLimitListeners.push(callback);
  }

  get settings(): AppSettings {
    return this._settings;
  }

  static async getSettingsFilePath(): Promise<string> {
    const dataDir = await appDataDir();
    return `${dataDir}\\settings.json`;
  }

  private static loadSettings = async (): Promise<AppSettings> => {
    const defaults = await SettingsStore.getDefaultSettings();

    try {
      const content = await invoke<string>("read_settings_file", {
        path: await this.getSettingsFilePath(),
      });

      if (content) {
        const savedSettings = JSON.parse(content) as Partial<AppSettings>;

        return { ...defaults, ...savedSettings };
      } else {
        return defaults;
      }
    } catch (error) {
      console.error("Failed to load settings:", error);
    }

    return defaults;
  };

  static getDefaultSettings = async (): Promise<AppSettings> => {
    let home: string = "C:\\Users\\Documents";
    try {
      home = await homeDir();
    } catch (error) {
      console.error("Failed to get home directory:", error);
    }
    return {
      replayDownloadPath: `${home}\\StarCraft\\Maps\\Replays\\CWAL`,
      mapDownloadPath: `${home}\\StarCraft\\Maps\\CWAL`,
      hideShortReplays: true,
      maxApiRequestsTps: 10,
    };
  };

  updateReplayPath = async (path: string) => {
    try {
      this._settings.replayDownloadPath = path;
      await this.saveSettings();
      toast.success("Replay download path updated");
    } catch (error) {
      console.error("Failed to update replay path:", error);
      toast.error("Failed to update replay download path");
    }
  };

  updateMapPath = async (path: string) => {
    try {
      this._settings.mapDownloadPath = path;
      await this.saveSettings();
      toast.success("Map download path updated");
    } catch (error) {
      console.error("Failed to update map path:", error);
      toast.error("Failed to update map download path");
    }
  };

  updateHideShortReplays = async (hideShortReplays: boolean) => {
    try {
      this._settings.hideShortReplays = hideShortReplays;
      await this.saveSettings();
      toast.success("Profile viewing preferences updated");
    } catch (error) {
      console.error("Failed to update profile viewing preferences:", error);
      toast.error("Failed to update profile viewing preferences");
    }
  };

  updateMaxApiRequestsTps = async (value: number) => {
    try {
      this._settings.maxApiRequestsTps = value;
      await this.saveSettings();
      this.rateLimitListeners.forEach((cb) => cb(value));
      toast.success("API rate preference updated");
    } catch (error) {
      console.error("Failed to update API TPS:", error);
      toast.error("Failed to update API rate preference");
    }
  };

  resetToDefaults = async () => {
    const defaults = await SettingsStore.getDefaultSettings();
    this._settings = defaults;
    await this.saveSettings();
    toast.success("Settings reset to defaults");
  };

  private saveSettings = async () => {
    try {
      const content = JSON.stringify(this._settings, null, 2);
      await invoke("write_settings_file", {
        path: await SettingsStore.getSettingsFilePath(),
        content,
      });
    } catch (error) {
      console.error("Failed to save settings:", error);
    }
  };
}

let settingsStore: SettingsStore;

export const getSettingsStore = async (): Promise<SettingsStore> => {
  if (!settingsStore) {
    settingsStore = await SettingsStore.create();
  }
  return settingsStore;
};
