<script lang="ts">
  import { type UnlistenFn } from "@tauri-apps/api/event";
  import { onDestroy } from "svelte";
  import Input from "$lib/components/ui/input/input.svelte";
  import { getGb } from "$lib/scApi.svelte";
  import { configureReceiveTauriEvents } from "$lib/scrState.svelte";
  import type { GravaticBooster, PlayerSearchResult } from "gravatic-booster";
  import { setMode } from "mode-watcher";
  import "../app.css";
  import * as Command from "$lib/components/ui/command";

  setMode("dark");

  let port: number | null = $state(null);
  let unlisten: Promise<UnlistenFn> | null = null;
  let gb: Promise<GravaticBooster>;

  $effect.pre(() => {
    unlisten = configureReceiveTauriEvents();
    gb = getGb();
  });

  onDestroy(async () => {
    if (unlisten) {
      (await unlisten)();
    }
  });

  let searchResults: PlayerSearchResult[] = $state([]);
  let searchValue: string = $state("");
  let isInputFocused: boolean = $state(false);

  const playerSearch = async (searchValue: string) => {
    const _gb = await gb;
    console.log(searchValue);
    const current = await _gb.currentSeason();
    console.log(current);
    try {
      searchResults = await _gb.playerSearch(searchValue);
    } catch (e) {
      searchResults = [];
      console.error(e);
    }
  };

  const handleKeydown = (event: KeyboardEvent) => {
    if (event.key === "Escape") {
      isInputFocused = false;
    }
  };

  $effect(() => {
    searchResults.forEach(console.log);
  });

  $effect(() => {
    console.log("doing player search");
    playerSearch(searchValue);
  });
</script>

<svelte:head>
  <title>Home</title>
  <meta name="description" content="Svelte demo app" />
</svelte:head>

<div class="dropdown">
  <Command.Root
    class="rounded-lg border shadow-md md:min-w-[450px]"
    shouldFilter={false}
  >
    <Command.Input
      placeholder="Player Search"
      bind:value={searchValue}
      onfocus={() => (isInputFocused = true)}
      onblur={() => (isInputFocused = false)}
      onkeydown={handleKeydown}
    />
    <Command.List>
      {#if isInputFocused}
        <Command.Group>
          {#each searchResults as searchResult}
            <Command.Item>
              {searchResult.name}
            </Command.Item>
          {/each}
        </Command.Group>
      {/if}
    </Command.List>
  </Command.Root>
</div>

<section></section>

<style>
</style>
