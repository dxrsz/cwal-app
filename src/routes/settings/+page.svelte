<script lang="ts">
  import { Check, Clock, RotateCcw } from "@lucide/svelte";

  import { Badge } from "@/lib/components/ui/badge";
  import { Button } from "@/lib/components/ui/button";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "@/lib/components/ui/card";
  import { Input } from "@/lib/components/ui/input";
  import { getSettingsStore } from "@/lib/settingsStore.svelte";

  const settingsStore = getSettingsStore();

  let replayPath = $state("");
  let mapPath = $state("");

  let resolvedDefaults = $state<{
    replayDownloadPath: string;
    mapDownloadPath: string;
  } | null>(null);

  let replayPathTimer: number | null = null;
  let mapPathTimer: number | null = null;

  let replayPathSaved = $state(true);
  let mapPathSaved = $state(true);

  $effect(() => {
    if (settingsStore.initialized) {
      replayPath = settingsStore.settings.replayDownloadPath;
      mapPath = settingsStore.settings.mapDownloadPath;

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
      replayPathSaved = false;

      if (replayPathTimer !== null) {
        clearTimeout(replayPathTimer);
      }

      replayPathTimer = setTimeout(async () => {
        if (replayPath.trim()) {
          await settingsStore.updateReplayPath(replayPath.trim());
          replayPathSaved = true;
        }
      }, 500);
    }
  });

  $effect(() => {
    if (
      settingsStore.initialized &&
      mapPath !== settingsStore.settings.mapDownloadPath
    ) {
      mapPathSaved = false;

      if (mapPathTimer !== null) {
        clearTimeout(mapPathTimer);
      }

      mapPathTimer = setTimeout(async () => {
        if (mapPath.trim()) {
          await settingsStore.updateMapPath(mapPath.trim());
          mapPathSaved = true;
        }
      }, 500);
    }
  });

  const resetToDefaults = async () => {
    await settingsStore.resetToDefaults();
    replayPath = settingsStore.settings.replayDownloadPath;
    mapPath = settingsStore.settings.mapDownloadPath;
    replayPathSaved = true;
    mapPathSaved = true;
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

  const getStatusBadge = (isSaving: boolean, isSaved: boolean) => {
    if (isSaving) {
      return { variant: "secondary" as const, icon: Clock, text: "Saving..." };
    } else if (isSaved) {
      return { variant: "default" as const, icon: Check, text: "Saved" };
    } else {
      return { variant: "outline" as const, icon: Clock, text: "Pending" };
    }
  }
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
            {#if settingsStore.initialized}
              {@const status = getStatusBadge(
                settingsStore.savingReplayPath,
                replayPathSaved,
              )}
              <Badge variant={status.variant} class="flex items-center gap-1">
                <status.icon class="size-3" />
                {status.text}
              </Badge>
            {/if}
          </div>
        </div>
        <Input
          id="replay-path"
          bind:value={replayPath}
          placeholder="C:\Users\YourName\Documents\StarCraft\Maps\Replays\CWAL"
        />
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
            {#if settingsStore.initialized}
              {@const status = getStatusBadge(
                settingsStore.savingMapPath,
                mapPathSaved,
              )}
              <Badge variant={status.variant} class="flex items-center gap-1">
                <status.icon class="size-3" />
                {status.text}
              </Badge>
            {/if}
          </div>
        </div>
        <Input
          id="map-path"
          bind:value={mapPath}
          placeholder="C:\Users\YourName\Documents\StarCraft\Maps\CWAL"
        />
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

  {#if !settingsStore.initialized}
    <Card>
      <CardContent class="p-6">
        <p class="text-muted-foreground">Loading settings...</p>
      </CardContent>
    </Card>
  {/if}
</div>
