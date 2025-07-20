<script lang="ts">
  import "@/app.css";

  import { onMount } from "svelte";

  import { goto, onNavigate } from "$app/navigation";
  import type { GravaticBooster, PlayerSearchResult } from "gravatic-booster";

  import * as Avatar from "@/lib/components/ui/avatar";
  import { Badge } from "@/lib/components/ui/badge";
  import * as Command from "@/lib/components/ui/command";
  import { getGb } from "@/lib/scApi.svelte";
  import { avatarOrDefault, debounce } from "@/lib/utils";

  interface Props {
    onPlayerSelect?: (name: string, gateway: string) => void;
  }

  const { onPlayerSelect }: Props = $props();

  let searching: boolean = $state(false);
  let gb: Promise<GravaticBooster> = getGb();
  let inputHeight: number = $state(0);
  let gatewayNames: Map<number, string> = $state(new Map());

  const resetSearching = () => {
    searching = false;
    searchResults = [];
    searchValue = "";
  };

  onMount(resetSearching);
  onNavigate(resetSearching);

  let searchResults: PlayerSearchResult[] = $state([]);
  let searchValue: string = $state("");

  const loadGatewayInfo = async () => {
    try {
      const _gb = await gb;
      const gateways = _gb.gateways();
      const gatewayMap = new Map<number, string>();

      for (const gateway of gateways) {
        gatewayMap.set(gateway.id, gateway.name);
      }
      gatewayNames = gatewayMap;
    } catch (e) {
      console.error("Failed to load gateway info:", e);
    }
  };

  $effect(() => {
    loadGatewayInfo();
  });

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
    if (onPlayerSelect) {
      onPlayerSelect(name, gateway);
    } else {
      // Default behavior: navigate to player page
      goto(`/player/${gateway}/${encodeURIComponent(name)}`);
    }
    resetSearching();
  };

  const getGatewayName = (gatewayId: number): string => {
    return gatewayNames.get(gatewayId) || `Gateway ${gatewayId}`;
  };

  const formatPoints = (points: number): string => {
    return points.toLocaleString();
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
        <Command.Input
          placeholder="ID or Battle Tag"
          bind:value={searchValue}
        />
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
                  class="cursor-pointer p-4"
                  value="{searchResult.name}@{searchResult.gatewayId}"
                  onSelect={() => {
                    console.log("Item selected:", searchResult);
                    handlePlayerSelect(
                      searchResult.name,
                      `${searchResult.gatewayId}`,
                    );
                  }}
                >
                  <div
                    class="flex flex-row justify-between items-center w-full gap-4"
                  >
                    <div class="flex flex-row items-center gap-3">
                      <Avatar.Root class="w-8 h-8">
                        <Avatar.Image
                          src={avatarOrDefault(searchResult.avatar)}
                        />
                        <Avatar.Fallback
                          >{searchResult.name
                            .slice(0, 2)
                            .toUpperCase()}</Avatar.Fallback
                        >
                      </Avatar.Root>

                      <div class="flex flex-col gap-1">
                        <div class="flex flex-row items-center gap-2">
                          <span class="font-medium leading-none">
                            {searchResult.name}
                          </span>
                          {#if searchResult.rank}{/if}
                        </div>
                        <div
                          class="flex flex-row items-center gap-2 text-sm text-muted-foreground"
                        >
                          <span>{searchResult.battletag}</span>
                          <span>•</span>
                          <span>{getGatewayName(searchResult.gatewayId)}</span>
                        </div>
                      </div>
                    </div>

                    <div class="flex flex-col items-end gap-1">
                      <div class="flex items-center gap-2">
                        <Badge variant="outline" class="text-xs">
                          Rank #{searchResult.rank}
                        </Badge>
                      </div>
                      {#if searchResult.points}
                        <span class="text-sm font-medium text-muted-foreground">
                          {formatPoints(searchResult.points)} MMR
                        </span>
                      {/if}
                    </div>
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
