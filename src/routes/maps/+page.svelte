<script lang="ts">
  import "@/app.css";

  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { toast } from "svelte-sonner";

  import type { GravaticBooster } from "gravatic-booster";

  import * as Select from "@/lib/components/ui/select";
  import * as Table from "@/lib/components/ui/table";
  import { Button } from "@/lib/components/ui/button";
  import { getGb } from "@/lib/scApi.svelte";
  import { getSettingsStore } from "@/lib/settingsStore.svelte";

  let gb: Promise<GravaticBooster> = getGb();
  const settingsStore = getSettingsStore();

  let currentSeason: number = $state(0);
  let season: string | undefined = $state(undefined);
  let maps: Awaited<ReturnType<typeof GravaticBooster.prototype.maps>> = $state([]);
  let downloadingMaps = $state(new Set<string>());

  const seasons = $derived(
    Array(currentSeason)
      .keys()
      .map((n) => n + 1)
      .toArray()
      .reverse(),
  );

  const downloadMap = async (map: { url: string; fileName: string; seasonId: number }) => {
    if (!settingsStore.initialized) {
      toast.error("Settings not loaded yet. Please try again.");
      return;
    }

    const mapKey = `${map.fileName}_${map.url}`;
    if (downloadingMaps.has(mapKey)) {
      return;
    }

    downloadingMaps.add(mapKey);
    downloadingMaps = new Set(downloadingMaps);

    try {
      const seasonDirectory = `${settingsStore.settings.mapDownloadPath}\\Season ${map.seasonId}`;
      const result = await invoke<string>("download_file", {
        url: map.url,
        destinationPath: seasonDirectory,
        filename: map.fileName
      });
      
      toast.success(`Map downloaded successfully`, {
        description: `Saved to: ${result}`
      });
    } catch (error) {
      console.error("Download failed:", error);
      toast.error("Download failed", {
        description: String(error)
      });
    } finally {
      downloadingMaps.delete(mapKey);
      downloadingMaps = new Set(downloadingMaps);
    }
  };

  onMount(async () => {
    const _gb = await gb;
    currentSeason = await _gb.currentSeason();
    season = `${currentSeason}`;
    maps = await _gb.maps();
  });
</script>

<div class="p-6 space-y-6">
  <div class="flex items-center justify-between">
    <div>
      <h1 class="text-2xl font-bold">Maps</h1>
      <p class="text-muted-foreground">StarCraft: Remastered Official Ladder Maps</p>
    </div>
    <div class="flex items-center gap-2">
      <label class="text-sm font-medium">Season:</label>
      <Select.Root type="single" name="season" bind:value={season}>
        <Select.Trigger class="w-[140px]">
          {season != null ? `Season ${season}` : "Select Season"}
        </Select.Trigger>
        <Select.Content>
          <Select.Group>
            {#each seasons as season}
              <Select.Item value={`${season}`} label={`Season ${season}`}>
                {`Season ${season}`}
              </Select.Item>
            {/each}
          </Select.Group>
        </Select.Content>
      </Select.Root>
    </div>
  </div>

  <Table.Root>
    <Table.Header>
      <Table.Row>
        <Table.Head class="w-[100px]">Name</Table.Head>
        <Table.Head>Version</Table.Head>
        <Table.Head>Size</Table.Head>
        <Table.Head>Last Modified</Table.Head>
        <Table.Head class="text-right">Download</Table.Head>
      </Table.Row>
    </Table.Header>
    <Table.Body>
      {#each maps as map}
        {#if `${map.seasonId}` === season}
          <Table.Row>
            <Table.Cell class="font-medium">{map.fileName}</Table.Cell>
            <Table.Cell>{map.version}</Table.Cell>
            <Table.Cell>{map.width}x{map.height}</Table.Cell>
            <Table.Cell
              >{new Date(map.modified_epoch).toLocaleDateString()}</Table.Cell
            >
            <Table.Cell class="text-right">
              {@const mapKey = `${map.fileName}_${map.url}`}
              <Button 
                onclick={() => downloadMap(map)}
                disabled={downloadingMaps.has(mapKey)}
                size="sm"
                variant="outline"
                class="cursor-pointer"
              >
                {downloadingMaps.has(mapKey) ? "Downloading..." : "Download"}
              </Button>
            </Table.Cell>
          </Table.Row>
        {/if}
      {/each}
    </Table.Body>
  </Table.Root>
</div>
