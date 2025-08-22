import { invoke } from '@tauri-apps/api/core';
import { appDataDir, homeDir } from '@tauri-apps/api/path';
import { toast } from 'svelte-sonner';

export interface AppSettings {
    replayDownloadPath: string;
    mapDownloadPath: string;
    hideShortReplays: boolean;
}

export class SettingsStore {
    private constructor(
        private _settings: AppSettings
    ) { }

    static create = async () => {
        const settings = await SettingsStore.loadSettings();
        const store = new SettingsStore(
            settings
        );
        return store;
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
            const content = await invoke<string>('read_settings_file', {
                path: await this.getSettingsFilePath()
            });

            if (content) {
                const savedSettings = JSON.parse(content) as Partial<AppSettings>;

                return { ...defaults, ...savedSettings };
            } else {
                return defaults;
            }
        } catch (error) {
            console.error('Failed to load settings:', error);
        }

        return defaults;
    }

    static getDefaultSettings = async (): Promise<AppSettings> => {
        try {
            const home = await homeDir();
            return {
                replayDownloadPath: `${home}\\Documents\\StarCraft\\Maps\\Replays\\CWAL`,
                mapDownloadPath: `${home}\\Documents\\StarCraft\\Maps\\CWAL`,
                hideShortReplays: true
            };
        } catch (error) {
            console.error('Failed to get home directory:', error);
            return {
                replayDownloadPath: 'C:\\Users\\Documents\\StarCraft\\Maps\\Replays\\CWAL',
                mapDownloadPath: 'C:\\Users\\Documents\\StarCraft\\Maps\\CWAL',
                hideShortReplays: true
            };
        }
    }

    updateReplayPath = async (path: string) => {
        try {
            this._settings.replayDownloadPath = path;
            await this.saveSettings();
            toast.success('Replay download path updated');
        } catch (error) {
            console.error('Failed to update replay path:', error);
            toast.error('Failed to update replay download path');
        }
    }

    updateMapPath = async (path: string) => {
        try {
            this._settings.mapDownloadPath = path;
            await this.saveSettings();
            toast.success('Map download path updated');
        } catch (error) {
            console.error('Failed to update map path:', error);
            toast.error('Failed to update map download path');
        }
    }

    updateHideShortReplays = async (hideShortReplays: boolean) => {
        try {
            this._settings.hideShortReplays = hideShortReplays;
            await this.saveSettings();
            toast.success('Profile viewing preferences updated');
        } catch (error) {
            console.error('Failed to update profile viewing preferences:', error);
            toast.error('Failed to update profile viewing preferences');
        }
    }

    private saveSettings = async () => {
        try {
            const content = JSON.stringify(this._settings, null, 2);
            await invoke('write_settings_file', {
                path: await SettingsStore.getSettingsFilePath(),
                content
            });
        } catch (error) {
            console.error('Failed to save settings:', error);
        }
    }
}

let settingsStore: SettingsStore;

export const getSettingsStore = async (): Promise<SettingsStore> => {
    if (!settingsStore) {
        settingsStore = await SettingsStore.create();
    }
    return settingsStore;
}
