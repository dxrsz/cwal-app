<script lang="ts">
  import "@/app.css";

  import { getGb } from "@/lib/scApi.svelte";
  import { onMount } from "svelte";

  import type { GravaticBooster, PlayerSearchResult } from "gravatic-booster";
  import * as Select from "@/lib/components/ui/select";
  import * as Table from "$lib/components/ui/table";

  let gb: Promise<GravaticBooster> = getGb();

  let currentSeason: number = $state(0);
  let season: string | undefined = $state(undefined);
  let maps: Awaited<ReturnType<typeof GravaticBooster.prototype.maps>> = $state(
    [],
  );

  const seasons = $derived(
    Array(currentSeason)
      .keys()
      .map((n) => n + 1)
      .toArray()
      .reverse(),
  );

  onMount(async () => {
    const _gb = await gb;
    currentSeason = await _gb.currentSeason();
    season = `${currentSeason}`;
    maps = await _gb.maps();
  });
</script>

<div class="p-2">
  <Select.Root type="single" name="season" bind:value={season}>
    <Select.Trigger class="w-[180px]"
      >{season != null ? `Season ${season}` : "Season"}</Select.Trigger
    >
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

  <Table.Root>
    <Table.Caption>StarCraft: Remastered Official Ladder Maps</Table.Caption>
    <Table.Header>
      <Table.Row>
        <Table.Head class="w-[100px]">Name</Table.Head>
        <Table.Head>Version</Table.Head>
        <Table.Head>Size</Table.Head>
        <Table.Head>Last Modified</Table.Head>
        <Table.Head class="text-right">Download Link</Table.Head>
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
            <Table.Cell class="text-right"
              ><a href={map.url}>Download</a></Table.Cell
            >
          </Table.Row>
        {/if}
      {/each}
    </Table.Body>
  </Table.Root>
</div>
