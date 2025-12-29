<script lang="ts">
  import { onMount } from "svelte";

  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import User from "@lucide/svelte/icons/user";
  import TimeAgo from "javascript-time-ago";
  import { toast } from "svelte-sonner";

  import Race from "@/lib/components/icons/race.svelte";
  import * as Avatar from "@/lib/components/ui/avatar";
  import Button from "@/lib/components/ui/button/button.svelte";
  import * as Card from "@/lib/components/ui/card";
  import {
    type SavedPlayer,
    type SavedPlayersStore,
    getSavedPlayersStore,
  } from "@/lib/savedPlayersStore.svelte";
  import { getGb } from "@/lib/scApi.svelte";
  import { avatarOrDefault } from "@/lib/utils";

  let savedPlayersStore: SavedPlayersStore | null = $state(null);
  const timeAgo = new TimeAgo("en-US");

  onMount(async () => {
    savedPlayersStore = await getSavedPlayersStore();
  });

  let players = $derived(
    savedPlayersStore ? (savedPlayersStore as SavedPlayersStore).players : [],
  );

  const syncPlayer = async (player: SavedPlayer) => {
    const _gb = await getGb();
    const toastId = toast.loading(`Syncing ${player.alias}...`);
    try {
      for (const p of player.profiles) {
        try {
          const profile = await _gb.minimalAccountWithGamesPlayedLastWeek(
            p.toon,
            { gateway: p.gateway },
          );

          if (profile.auroraId !== player.auroraId) continue;

          const leaderboard = await _gb.leaderboard({
            seasonId: profile.currentSeason,
          });
          const acct = await profile.requestedProfile?.accountRankings(
            leaderboard.id,
          );

          if (acct && acct.rankings) {
            const newProfiles = acct.rankings.map((r) => ({
              toon: r.toon,
              gateway: r.gatewayId,
              lastViewed: Date.now(),
              race: r.featureRace,
              avatarUrl: avatarOrDefault(r.avatar),
            }));

            savedPlayersStore?.setProfiles(player.auroraId, newProfiles);
            toast.success("Synced profiles", { id: toastId });
            return;
          }
        } catch (e) {
          console.warn(`Failed to sync with profile ${p.toon}`, e);
        }
      }
      toast.error("Could not sync player. No valid profiles found.", {
        id: toastId,
      });
    } catch (e) {
      console.error("Sync error", e);
      toast.error("Error syncing player", { id: toastId });
    }
  };
</script>

<div class="h-full w-full overflow-y-auto p-6 space-y-6">
  <div class="flex items-center justify-between">
    <h1 class="text-3xl font-bold">Saved Players</h1>
  </div>

  {#if players.length === 0}
    <div class="text-center py-20 text-muted-foreground">
      <User class="w-12 h-12 mx-auto mb-4 opacity-50" />
      <p class="text-lg font-medium">No saved players yet</p>
      <p class="text-sm">Visit a player profile to save them</p>
    </div>
  {:else}
    <div class="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
      {#each players as player (player.auroraId)}
        <Card.Root>
          <Card.Header
            class="flex flex-row items-center justify-between space-y-0 pb-2"
          >
            <Card.Title class="text-xl font-bold">{player.alias}</Card.Title>
            <div class="flex items-center gap-1">
              <Button
                variant="ghost"
                size="icon"
                class="h-8 w-8 text-muted-foreground hover:text-foreground cursor-pointer"
                onclick={() => syncPlayer(player)}
              >
                <RefreshCw class="h-4 w-4" />
              </Button>
              <Button
                variant="ghost"
                size="icon"
                class="h-8 w-8 text-muted-foreground hover:text-destructive cursor-pointer"
                onclick={() => savedPlayersStore?.removePlayer(player.auroraId)}
              >
                <Trash2 class="h-4 w-4" />
              </Button>
            </div>
          </Card.Header>
          <Card.Content>
            <div class="space-y-4 pt-4">
              <div class="space-y-2">
                <span class="text-sm font-medium text-muted-foreground"
                  >Known Profiles</span
                >
                <div class="grid gap-2">
                  {#each player.profiles as profile}
                    <a
                      href={`/player/${profile.gateway}/${profile.toon}`}
                      class="flex items-center gap-3 p-2 rounded-md hover:bg-muted transition-colors border"
                    >
                      <Avatar.Root class="h-8 w-8 rounded-md">
                        <Avatar.Image
                          src={profile.avatarUrl}
                          alt={profile.toon}
                        />
                        <Avatar.Fallback class="text-xs"
                          >{profile.toon
                            .slice(0, 2)
                            .toUpperCase()}</Avatar.Fallback
                        >
                      </Avatar.Root>
                      <div class="flex-1 min-w-0">
                        <div class="flex items-center justify-between">
                          <div class="flex items-center gap-1.5 min-w-0">
                            <span class="font-medium truncate text-sm"
                              >{profile.toon}</span
                            >
                            {#if profile.race}
                              <div class="scale-75 origin-left">
                                <Race race={profile.race} />
                              </div>
                            {/if}
                          </div>
                          <span
                            class="text-xs text-muted-foreground flex-shrink-0 ml-2"
                            >Gateway {profile.gateway}</span
                          >
                        </div>
                        {#if profile.lastViewed}
                          <div class="text-[10px] text-muted-foreground">
                            Last viewed {timeAgo.format(profile.lastViewed)}
                          </div>
                        {/if}
                      </div>
                    </a>
                  {/each}
                </div>
              </div>
            </div>
          </Card.Content>
        </Card.Root>
      {/each}
    </div>
  {/if}
</div>
