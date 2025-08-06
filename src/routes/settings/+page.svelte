<script lang="ts">
  import { RotateCcw, FolderOpen } from "@lucide/svelte";
  import { open } from "@tauri-apps/plugin-dialog";

  import { Button } from "@/lib/components/ui/button";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "@/lib/components/ui/card";
  import { Input } from "@/lib/components/ui/input";
  import { Switch } from "@/lib/components/ui/switch";
  import { getSettingsStore } from "@/lib/settingsStore.svelte";

  const settingsStore = getSettingsStore();

  let replayPath = $state("");
  let mapPath = $state("");
  let hideShortReplays = $state(false);

  let resolvedDefaults = $state<{
    replayDownloadPath: string;
    mapDownloadPath: string;
  } | null>(null);

  let replayPathTimer: number | null = null;
  let mapPathTimer: number | null = null;

  $effect(() => {
    if (settingsStore.initialized) {
      replayPath = settingsStore.settings.replayDownloadPath;
      mapPath = settingsStore.settings.mapDownloadPath;
      hideShortReplays = settingsStore.settings.hideShortReplays;

      settingsStore.getResolvedDefaults().then((defaults) => {
        resolvedDefaults = defaults;
      });
    }
  });

  $effect(() => {
    if (
      settingsStore.initialized &&
      replayPath !== settingsStore.settings.replayDownloadPath
    ) {
      if (replayPathTimer !== null) {
        clearTimeout(replayPathTimer);
      }

      replayPathTimer = setTimeout(async () => {
        if (replayPath.trim()) {
          await settingsStore.updateReplayPath(replayPath.trim());
        }
      }, 500);
    }
  });

  $effect(() => {
    if (
      settingsStore.initialized &&
      mapPath !== settingsStore.settings.mapDownloadPath
    ) {
      if (mapPathTimer !== null) {
        clearTimeout(mapPathTimer);
      }

      mapPathTimer = setTimeout(async () => {
        if (mapPath.trim()) {
          await settingsStore.updateMapPath(mapPath.trim());
        }
      }, 500);
    }
  });

  $effect(() => {
    if (
      settingsStore.initialized &&
      hideShortReplays !== settingsStore.settings.hideShortReplays
    ) {
      settingsStore.updateHideShortReplays(hideShortReplays);
    }
  });

  const resetToDefaults = async () => {
    await settingsStore.resetToDefaults();
    replayPath = settingsStore.settings.replayDownloadPath;
    mapPath = settingsStore.settings.mapDownloadPath;
    hideShortReplays = settingsStore.settings.hideShortReplays;
  };

  const resetReplayPath = async () => {
    if (resolvedDefaults) {
      replayPath = resolvedDefaults.replayDownloadPath;
    }
  };

  const resetMapPath = async () => {
    if (resolvedDefaults) {
      mapPath = resolvedDefaults.mapDownloadPath;
    }
  };

  const selectReplayFolder = async () => {
    try {
      const selected = await open({
        directory: true,
        defaultPath: replayPath || undefined,
      });
      
      if (selected && typeof selected === 'string') {
        replayPath = selected;
      }
    } catch (error) {
      console.error('Failed to open folder picker:', error);
    }
  };

  const selectMapFolder = async () => {
    try {
      const selected = await open({
        directory: true,
        defaultPath: mapPath || undefined,
      });
      
      if (selected && typeof selected === 'string') {
        mapPath = selected;
      }
    } catch (error) {
      console.error('Failed to open folder picker:', error);
    }
  };
</script>

<div class="p-6 space-y-6">
  <div>
    <h1 class="text-2xl font-bold">Settings</h1>
    <p class="text-muted-foreground">
      Configure your download locations and preferences.
    </p>
  </div>

  <Card>
    <CardHeader>
      <CardTitle>Download Locations</CardTitle>
      <CardDescription>
        Set where replays and maps should be downloaded to.
      </CardDescription>
    </CardHeader>
    <CardContent class="space-y-4">
      <div class="space-y-2">
        <div class="flex items-center justify-between">
          <label class="text-sm font-medium" for="replay-path">
            Replay Download Path
          </label>
          <div class="flex items-center gap-2">
            <Button
              onclick={resetReplayPath}
              variant="ghost"
              size="sm"
              disabled={!resolvedDefaults ||
                replayPath === resolvedDefaults.replayDownloadPath}
              class="h-6 px-2 text-xs cursor-pointer"
            >
              <RotateCcw class="size-3 mr-1" />
              Reset
            </Button>
          </div>
        </div>
        <div class="flex gap-2">
          <Input
            id="replay-path"
            bind:value={replayPath}
            placeholder="C:\Users\YourName\Documents\StarCraft\Maps\Replays\CWAL"
            class="flex-1"
          />
          <Button
            onclick={selectReplayFolder}
            variant="outline"
            size="sm"
            class="px-3"
            title="Browse for folder"
          >
            <FolderOpen class="size-4" />
          </Button>
        </div>
        <p class="text-xs text-muted-foreground">
          {#if resolvedDefaults}
            Default: {resolvedDefaults.replayDownloadPath}
          {:else}
            Default: Loading...
          {/if}
        </p>
      </div>

      <div class="space-y-2">
        <div class="flex items-center justify-between">
          <label class="text-sm font-medium" for="map-path">
            Map Download Path
          </label>
          <div class="flex items-center gap-2">
            <Button
              onclick={resetMapPath}
              variant="ghost"
              size="sm"
              disabled={!resolvedDefaults ||
                mapPath === resolvedDefaults.mapDownloadPath}
              class="h-6 px-2 text-xs cursor-pointer"
            >
              <RotateCcw class="size-3 mr-1" />
              Reset
            </Button>
          </div>
        </div>
        <div class="flex gap-2">
          <Input
            id="map-path"
            bind:value={mapPath}
            placeholder="C:\Users\YourName\Documents\StarCraft\Maps\CWAL"
            class="flex-1"
          />
          <Button
            onclick={selectMapFolder}
            variant="outline"
            size="sm"
            class="px-3"
            title="Browse for folder"
          >
            <FolderOpen class="size-4" />
          </Button>
        </div>
        <p class="text-xs text-muted-foreground">
          {#if resolvedDefaults}
            Default: {resolvedDefaults.mapDownloadPath}
          {:else}
            Default: Loading...
          {/if}
        </p>
      </div>

      <div class="pt-4 border-t">
        <Button
          onclick={resetToDefaults}
          variant="outline"
          size="sm"
          class="cursor-pointer"
        >
          <RotateCcw class="size-3 mr-1" />
          Reset All to Defaults
        </Button>
      </div>
    </CardContent>
  </Card>

  <Card>
    <CardHeader>
      <CardTitle>Replay Preferences</CardTitle>
      <CardDescription>
        Configure how replays are organized and handled.
      </CardDescription>
    </CardHeader>
    <CardContent class="space-y-4">
      <div class="flex items-center justify-between">
        <div class="space-y-1">
          <label class="text-sm font-medium" for="hide-short-replays">
            Hide short replays
          </label>
          <p class="text-xs text-muted-foreground">
            Automatically move replays shorter than 1 minute to a hidden directory
          </p>
        </div>
        <Switch
          id="hide-short-replays"
          bind:checked={hideShortReplays}
          class="cursor-pointer"
        />
      </div>
    </CardContent>
  </Card>

  {#if !settingsStore.initialized}
    <Card>
      <CardContent class="p-6">
        <p class="text-muted-foreground">Loading settings...</p>
      </CardContent>
    </Card>
  {/if}
</div>
