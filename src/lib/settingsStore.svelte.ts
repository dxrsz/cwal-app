import { invoke } from '@tauri-apps/api/core';
import { appDataDir, homeDir } from '@tauri-apps/api/path';

export interface AppSettings {
    replayDownloadPath: string;
    mapDownloadPath: string;
}

class SettingsStore {
    private _settings = $state<AppSettings>({
        replayDownloadPath: '',
        mapDownloadPath: ''
    });

    private _initialized = $state(false);
    private _savingReplayPath = $state(false);
    private _savingMapPath = $state(false);
    private settingsFilePath: string = '';

    constructor() {
        this.initialize();
    }

    get settings(): AppSettings {
        return this._settings;
    }

    get initialized(): boolean {
        return this._initialized;
    }

    get savingReplayPath(): boolean {
        return this._savingReplayPath;
    }

    get savingMapPath(): boolean {
        return this._savingMapPath;
    }

    private initialize = async () => {
        try {
            const appData = await appDataDir();
            this.settingsFilePath = `${appData}settings.json`;

            await this.loadSettings();
            this._initialized = true;
        } catch (error) {
            console.error('Failed to initialize settings store:', error);
            await this.setDefaults();
            this._initialized = true;
        }
    }

    private loadSettings = async () => {
        try {
            const content = await invoke<string>('read_settings_file', {
                path: this.settingsFilePath
            });

            if (content) {
                const savedSettings = JSON.parse(content) as Partial<AppSettings>;

                const defaults = await this.getDefaultSettings();
                this._settings = { ...defaults, ...savedSettings };
            } else {
                await this.setDefaults();
            }
        } catch (error) {
            console.error('Failed to load settings:', error);
            await this.setDefaults();
        }
    }

    private setDefaults = async () => {
        this._settings = await this.getDefaultSettings();
        await this.saveSettings();
    }

    private getDefaultSettings = async (): Promise<AppSettings> => {
        try {
            const home = await homeDir();
            return {
                replayDownloadPath: `${home}\\Documents\\StarCraft\\Maps\\Replays\\CWAL`,
                mapDownloadPath: `${home}\\Documents\\StarCraft\\Maps\\CWAL`
            };
        } catch (error) {
            console.error('Failed to get home directory:', error);
            return {
                replayDownloadPath: 'C:\\Users\\Documents\\StarCraft\\Maps\\Replays\\CWAL',
                mapDownloadPath: 'C:\\Users\\Documents\\StarCraft\\Maps\\CWAL'
            };
        }
    }

    updateReplayPath = async (path: string) => {
        this._savingReplayPath = true;
        try {
            this._settings.replayDownloadPath = path;
            await this.saveSettings();
        } finally {
            this._savingReplayPath = false;
        }
    }

    updateMapPath = async (path: string) => {
        this._savingMapPath = true;
        try {
            this._settings.mapDownloadPath = path;
            await this.saveSettings();
        } finally {
            this._savingMapPath = false;
        }
    }

    updateSettings = async (newSettings: Partial<AppSettings>) => {
        this._settings = { ...this._settings, ...newSettings };
        await this.saveSettings();
    }

    private saveSettings = async () => {
        try {
            const content = JSON.stringify(this._settings, null, 2);
            await invoke('write_settings_file', {
                path: this.settingsFilePath,
                content
            });
        } catch (error) {
            console.error('Failed to save settings:', error);
        }
    }

    resetToDefaults = async () => {
        await this.setDefaults();
    }

    getResolvedDefaults = async (): Promise<AppSettings> => {
        return await this.getDefaultSettings();
    }
}

let settingsStore: SettingsStore;

export const getSettingsStore = (): SettingsStore => {
    if (!settingsStore) {
        settingsStore = new SettingsStore();
    }
    return settingsStore;
}
