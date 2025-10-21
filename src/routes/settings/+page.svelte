<script lang="ts">
  import { onMount } from "svelte";

  import { FolderOpen, RotateCcw } from "@lucide/svelte";
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
  import {
    type AppSettings,
    SettingsStore,
    getSettingsStore,
  } from "@/lib/settingsStore.svelte";
  import { debounce } from "@/lib/utils";

  const settingsStorePromise = getSettingsStore();

  let replayPath = $state("");
  let mapPath = $state("");
  let maxApiRequestsTps = $state(0);
  let maxReplayDownloadsTps = $state(0);
  let maxApiRequestsTpsInput = $state("");
  let maxReplayDownloadsTpsInput = $state("");
  let initialized = $state(false);

  let resolvedDefaults = $state<AppSettings | null>(null);

  onMount(async () => {
    resolvedDefaults = await SettingsStore.getDefaultSettings();
    const { settings } = await settingsStorePromise;
    replayPath = settings.replayDownloadPath;
    mapPath = settings.mapDownloadPath;
    maxApiRequestsTps =
      settings.maxApiRequestsTps ?? resolvedDefaults.maxApiRequestsTps;
    maxReplayDownloadsTps =
      settings.maxReplayDownloadsTps ?? resolvedDefaults.maxReplayDownloadsTps;
    maxApiRequestsTpsInput = String(maxApiRequestsTps);
    maxReplayDownloadsTpsInput = String(maxReplayDownloadsTps);
    initialized = true;
  });

  const setReplayDownloadPath = debounce(async (path: string) => {
    const settingsStore = await settingsStorePromise;
    if (path !== settingsStore.settings.replayDownloadPath) {
      settingsStore.updateReplayPath(path);
    }
  }, 1000);

  $effect(() => {
    setReplayDownloadPath(replayPath);
  });

  const setMapDownloadPath = debounce(async (path: string) => {
    const settingsStore = await settingsStorePromise;
    if (path !== settingsStore.settings.mapDownloadPath) {
      settingsStore.updateMapPath(path);
    }
  }, 1000);

  $effect(() => {
    setMapDownloadPath(mapPath);
  });

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

      if (selected && typeof selected === "string") {
        replayPath = selected;
      }
    } catch (error) {
      console.error("Failed to open folder picker:", error);
    }
  };

  const selectMapFolder = async () => {
    try {
      const selected = await open({
        directory: true,
        defaultPath: mapPath || undefined,
      });

      if (selected && typeof selected === "string") {
        mapPath = selected;
      }
    } catch (error) {
      console.error("Failed to open folder picker:", error);
    }
  };

  const setMaxApiRequestsTps = debounce(async (val: number) => {
    const store = await settingsStorePromise;
    if (val !== store.settings.maxApiRequestsTps) {
      store.updateMaxApiRequestsTps(val);
    }
  }, 800);

  $effect(() => {
    if (!initialized) return;
    setMaxApiRequestsTps(maxApiRequestsTps);
  });

  $effect(() => {
    if (!initialized) return;
    const v = parseFloat(maxApiRequestsTpsInput);
    if (!isNaN(v)) maxApiRequestsTps = v;
    else
      maxApiRequestsTps = resolvedDefaults
        ? resolvedDefaults.maxApiRequestsTps
        : 0;
  });

  const setMaxReplayDownloadsTps = debounce(async (val: number) => {
    const store = await settingsStorePromise;
    if (val !== store.settings.maxReplayDownloadsTps) {
      store.updateMaxReplayDownloadsTps(val);
    }
  }, 800);

  $effect(() => {
    if (!initialized) return;
    setMaxReplayDownloadsTps(maxReplayDownloadsTps);
  });

  $effect(() => {
    if (!initialized) return;
    const v = parseFloat(maxReplayDownloadsTpsInput);
    if (!isNaN(v)) maxReplayDownloadsTps = v;
    else
      maxReplayDownloadsTps = resolvedDefaults
        ? resolvedDefaults.maxReplayDownloadsTps
        : 0;
  });
</script>

<div class="w-full h-[100vh] overflow-y-scroll scroll-smooth pb-8">
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
      </CardContent>
    </Card>

    <Card>
      <CardHeader>
        <CardTitle>Throughput Limits</CardTitle>
        <CardDescription
          >Optional soft limits (transactions per second).</CardDescription
        >
      </CardHeader>
      <CardContent class="space-y-4">
        <div class="space-y-2">
          <div class="flex items-center justify-between">
            <label class="text-sm font-medium" for="api-tps"
              >API Requests TPS</label
            >
            <Button
              variant="ghost"
              size="sm"
              class="h-6 px-2 text-xs cursor-pointer"
              onclick={() => {
                if (resolvedDefaults) {
                  maxApiRequestsTps = resolvedDefaults.maxApiRequestsTps;
                  maxApiRequestsTpsInput = String(
                    resolvedDefaults.maxApiRequestsTps,
                  );
                }
              }}
              disabled={!resolvedDefaults ||
                maxApiRequestsTps === resolvedDefaults?.maxApiRequestsTps}
            >
              <RotateCcw class="size-3 mr-1" />Reset
            </Button>
          </div>
          <div class="flex gap-2 items-center">
            <Input
              id="api-tps"
              type="number"
              min="0"
              step="0.1"
              class="w-32"
              bind:value={maxApiRequestsTpsInput}
            />
            <p class="text-xs text-muted-foreground">
              Default: {resolvedDefaults
                ? resolvedDefaults.maxApiRequestsTps
                : "…"}
            </p>
          </div>
        </div>

        <div class="space-y-2">
          <div class="flex items-center justify-between">
            <label class="text-sm font-medium" for="replay-tps"
              >Replay Downloads TPS</label
            >
            <Button
              variant="ghost"
              size="sm"
              class="h-6 px-2 text-xs cursor-pointer"
              onclick={() => {
                if (resolvedDefaults) {
                  maxReplayDownloadsTps =
                    resolvedDefaults.maxReplayDownloadsTps;
                  maxReplayDownloadsTpsInput = String(
                    resolvedDefaults.maxReplayDownloadsTps,
                  );
                }
              }}
              disabled={!resolvedDefaults ||
                maxReplayDownloadsTps ===
                  resolvedDefaults?.maxReplayDownloadsTps}
            >
              <RotateCcw class="size-3 mr-1" />Reset
            </Button>
          </div>
          <div class="flex gap-2 items-center">
            <Input
              id="replay-tps"
              type="number"
              min="0"
              step="0.05"
              class="w-32"
              bind:value={maxReplayDownloadsTpsInput}
            />
            <p class="text-xs text-muted-foreground">
              Default: {resolvedDefaults
                ? resolvedDefaults.maxReplayDownloadsTps
                : "…"}
            </p>
          </div>
        </div>
      </CardContent>
    </Card>
  </div>
</div>
