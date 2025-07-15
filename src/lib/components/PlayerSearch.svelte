<script lang="ts">
  import "@/app.css";

  import { onMount } from "svelte";
  import { goto, onNavigate } from "$app/navigation";
  import { getGb } from "@/lib/scApi.svelte";
  import { debounce } from "@/lib/utils";

  import * as Command from "@/lib/components/ui/command";
  import * as Avatar from "@/lib/components/ui/avatar";

  import type { GravaticBooster, PlayerSearchResult } from "gravatic-booster";

  let searching: boolean = $state(false);
  let gb: Promise<GravaticBooster> = getGb();
  let inputHeight: number = $state(0);

  const resetSearching = () => {
    searching = false;
    searchResults = [];
    searchValue = "";
  };

  onMount(resetSearching);
  onNavigate(resetSearching);

  let searchResults: PlayerSearchResult[] = $state([]);
  let searchValue: string = $state("");

  const playerSearch = debounce(async (searchValue: string) => {
    try {
      const _gb = await gb;
      if (!searchValue) {
        return;
      }
      searching = true;
      searchResults = await _gb.playerSearch(searchValue);
    } catch (e) {
      searchResults = [];
      console.error(e);
    } finally {
      searching = false;
    }
  }, 250);

  const handlePlayerSelect = (name: string, gateway: string) => {
    goto(`/player/${name}/${encodeURIComponent(gateway)}`);
    resetSearching();
  };

  $effect(() => {
    playerSearch(searchValue);
  });
</script>

<div class="w-full z-1 sticky top-0">
  <div class="absolute overflow-visible w-full">
    <Command.Root
      class="rounded-lg border shadow-md md:min-w-[450px]"
      shouldFilter={!searching}
    >
      <div bind:clientHeight={inputHeight} class="block">
        <Command.Input placeholder="Player Search" bind:value={searchValue} />
      </div>
      <Command.List>
        {#if searching}
          <Command.Group>
            <Command.Item>Loading</Command.Item>
          </Command.Group>
        {:else}
          {#key JSON.stringify(searchResults)}
            <Command.Group>
              {#each searchResults as searchResult}
                <Command.Item
                  class="cursor-pointer"
                  value="{searchResult.name}@{searchResult.gatewayId}"
                  onSelect={() => {
                    console.log("Item selected:", searchResult);
                    handlePlayerSelect(
                      searchResult.name,
                      `${searchResult.gatewayId}`,
                    );
                  }}
                >
                  <div class="flex flex-row justify-between w-full">
                    <div class="flex flex-col gap-1">
                      <medium class="font-medium leading-none">
                        {searchResult.name}
                      </medium>
                      <small class="text-sm leading-none"
                        >{searchResult.battletag}</small
                      >
                    </div>
                    {#if searchResult.avatar}
                      <Avatar.Root>
                        <Avatar.Image src={searchResult.avatar} />
                      </Avatar.Root>
                    {/if}
                  </div>
                </Command.Item>
              {/each}
            </Command.Group>
          {/key}
        {/if}
      </Command.List>
    </Command.Root>
  </div>
</div>
<div class="block" style="height: {inputHeight + 20}px;"></div>
